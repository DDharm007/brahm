from sqlalchemy.orm import DeclarativeBase

class Base(DeclarativeBase):
    pass

class AuditLog(Base):
    id = 1
    action = "login"
