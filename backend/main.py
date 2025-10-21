from fastapi import FastAPI, HTTPException
from playwright.sompo_scraper import login_and_get_session, get_quote

app = FastAPI()

@app.post("/api/sompo")
def sompo_teklif(payload: dict):
    try:
        session = login_and_get_session("BULUT1", "EEsigorta.2828")
        teklif = get_quote(payload["tc"], payload["plaka"], payload["model"])
        return {"ok": True, "teklif": teklif, "session": session}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))