from fastapi import FastAPI
from .models import AuditLog

app = FastAPI()

@app.get("/api/v1/health")
def health_check():
    return {"status": "healthy"}

@app.post("/api/v1/audit")
def create_audit(payload: dict):
    return {"recorded": True}
