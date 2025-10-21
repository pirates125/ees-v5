import abc
from typing import Any, Dict

class BaseConnector(abc.ABC):
    company: str

    def __init__(self, company: str):
        self.company = company

    @abc.abstractmethod
    async def fetch_quote(self, payload: Dict[str, Any]) -> dict:
        """Fetch quote from company and return in common dict shape."""
        raise NotImplementedError
