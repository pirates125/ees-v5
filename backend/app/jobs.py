import asyncio, uuid, datetime as dt
from typing import Dict, List
from .models import Quote
from .store import JOBS, QUOTES, REQUEST_TO_JOB
from .connectors.sompo import SompoConnector
from .connectors.anadolu import AnadoluConnector
from .connectors.axa import AxaConnector

CONNECTORS = [
    SompoConnector("Sompo"),
    AnadoluConnector("Anadolu"),
    AxaConnector("AXA"),
]

async def run_job(job_id: str, request_id: str, payload: dict):
    JOBS[job_id].update({"status": "running", "progress": 5, "message": "Başlatıldı"})
    errors: List[dict] = []

    async def fetch_one(connector):
        try:
            data = await connector.fetch_quote(payload)
            q = Quote(
                id=data["id"],
                requestId=request_id,
                company=data["company"],
                premium=float(data["premium"]),
                currency=data["currency"],
                validUntil=data.get("validUntil"),
                coverages=data.get("coverages", []),
                extras=data.get("extras", {}),
                createdAt=dt.datetime.utcnow().isoformat() + "Z",
            )
            return q, None
        except Exception as e:
            return None, {"company": connector.company, "reason": str(e)}

    tasks = [fetch_one(c) for c in CONNECTORS]
    for i, coro in enumerate(asyncio.as_completed(tasks), start=1):
        q, err = await coro
        JOBS[job_id]["progress"] = 5 + int(90 * i / max(1, len(tasks)))
        if q:
            QUOTES[request_id].append(q)
        if err:
            errors.append(err)

    status = "succeeded" if QUOTES[request_id] else "failed"
    if errors and QUOTES[request_id]:
        status = "partial"

    JOBS[job_id].update({
        "status": status,
        "progress": 100,
        "errors": errors,
        "screenshots": JOBS[job_id].get("screenshots", []),
    })
