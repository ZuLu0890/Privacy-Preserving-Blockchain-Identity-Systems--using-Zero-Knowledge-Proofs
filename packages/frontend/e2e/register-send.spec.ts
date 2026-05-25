import { test, expect } from "@playwright/test";

test.describe("Register flow", () => {
  test("shows register page with form", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h2")).toHaveText("Register Username");
    await expect(page.locator('input[placeholder="alice"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toHaveText("Register");
  });

  test("rejects short username", async ({ page }) => {
    await page.goto("/");
    const input = page.locator('input[placeholder="alice"]');
    await input.fill("ab");
    await page.locator('button[type="submit"]').click();
    await expect(page.locator("text=3")).toBeVisible();
  });
});

test.describe("Send flow", () => {
  test("shows send page with form", async ({ page }) => {
    await page.goto("/send");
    await expect(page.locator("h2")).toHaveText("Send Private Payment");
    await expect(page.locator('input[placeholder="alice"]')).toBeVisible();
    await expect(page.locator('input[type="number"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toHaveText("Send");
  });
});

test.describe("Navigation", () => {
  test("can navigate between register and send pages", async ({ page }) => {
    await page.goto("/");
    await page.locator('a[href="/send"]').click();
    await expect(page).toHaveURL("/send");
    await page.locator('a[href="/"]').click();
    await expect(page).toHaveURL("/");
  });
});

test.describe("Dark mode", () => {
  test("toggle button is visible", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Dark")).toBeVisible();
  });
});
