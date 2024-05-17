import { test, expect, type Page } from '@playwright/test';

const basePath = '/app';

test.beforeEach(async ({ page }) => {
    await page.goto(basePath);
});

test.describe('Login', () => {
    // test('protected request should return 401', async ({ page }) => {
    //     const protectedRequestPromise = await page.waitForResponse(basePath + '/protected');

    //     expect(protectedRequestPromise.status()).toBe(401);
    // });

    test('login button should redirect to /auth', async ({ page }) => {
        const loginButton = page.getByRole('link', { name: 'Log in' });

        const pagePromise = page.waitForEvent('popup');
        await loginButton.click();
        const popup = await pagePromise;

        const popupURL = new URL(popup.url());
        expect(popupURL.pathname).toContain('/auth');
    });
});