from pydantic import BaseModel
from typing import List, Optional, Literal, Dict

Currency = Literal["TRY", "USD", "EUR"]

class CoverageItem(BaseModel):
    code: str
    label: str
    limit: Optional[str] = None
    deductible: Optional[str] = None

class Quote(BaseModel):
    id: str
    requestId: str
    company: str
    premium: float
    currency: Currency
    validUntil: Optional[str] = None
    coverages: List[CoverageItem] = []
    extras: Dict[str, object] = {}
    createdAt: str

class CreatePayload(BaseModel):
    plate: str
    product: Literal["trafik","kasko","konut","saglik","tamamlayici","imm"]
    identity: Optional[str] = None
    phone: Optional[str] = None
    email: Optional[str] = None
    birthDate: Optional[str] = None

JobStatus = Literal["queued","running","succeeded","failed","partial"]

class JobStatusResponse(BaseModel):
    jobId: str
    status: JobStatus
    progress: Optional[int] = 0
    message: Optional[str] = None
    screenshots: Optional[List[Dict[str, str]]] = []
    errors: Optional[List[Dict[str, str]]] = []
