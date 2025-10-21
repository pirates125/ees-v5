import os, uuid, datetime as dt
from typing import Any, Dict
from .base import BaseConnector
from ..browser import browser_context
from ..utils import parse_tl
from playwright.async_api import TimeoutError as PWTimeout

class AxaConnector(BaseConnector):
    async def fetch_quote(self, payload: Dict[str, Any]) -> dict:
        url = os.getenv("AXA_URL", "https://example.com/axa/login")
        user = os.getenv("AXA_USER", "")
        pwd  = os.getenv("AXA_PASS", "")
        proxy= os.getenv("HTTP_PROXY") or None
        headless = os.getenv("PLAYWRIGHT_HEADLESS","true").lower() != "false"

        async with browser_context(proxy, headless=headless) as ctx:
            page = await ctx.new_page()
            try:
                await page.goto(url, timeout=30000)
                await page.fill('input[name="user"]', user)
                await page.fill('input[name="pass"]', pwd)
                await page.click('button[type="submit"]')
                await page.wait_for_load_state("networkidle", timeout=20000)

                plate = payload.get("plate","34ABC123")
                await page.fill('input[name="plate"]', plate)
                await page.click('button[type="submit"]')
                await page.wait_for_selector('.result .price', timeout=30000)
                price_text = await page.text_content('.result .price')
                premium = parse_tl(price_text or "4490")
                coverages = [{"code":"TRAFIK_ZORUNLU","label":"Zorunlu Trafik"}]
            except PWTimeout as e:
                raise RuntimeError(f"AXA timeout: {e}")
            finally:
                await page.close()

        return {
            "id": str(uuid.uuid4()),
            "company": self.company,
            "premium": float(premium),
            "currency": "TRY",
            "validUntil": (dt.datetime.utcnow() + dt.timedelta(hours=2)).isoformat() + "Z",
            "coverages": coverages,
            "extras": {"note":"axa playwright"},
        }
