import os
from contextlib import asynccontextmanager
from typing import Optional, Dict, Any
from playwright.async_api import async_playwright

@asynccontextmanager
async def browser_context(proxy_url: Optional[str] = None, headless: bool = True):
    async with async_playwright() as p:
        launch_args: Dict[str, Any] = {
            "headless": headless,
            "args": [
                "--disable-blink-features=AutomationControlled",
                "--no-sandbox",
                "--disable-dev-shm-usage",
            ],
        }
        if proxy_url:
            launch_args["proxy"] = {"server": proxy_url}
        browser = await p.chromium.launch(**launch_args)
        context = await browser.new_context(
            viewport={"width": 1366, "height": 768},
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0 Safari/537.36",
        )
        try:
            yield context
        finally:
            await context.close()
            await browser.close()
