import { test, expect, type Page } from '@playwright/test';

const basePath = '/auth';

test.beforeEach(async ({ page }) => {
    await page.goto(basePath);
});

test.describe('Login', () => {
    test('login form should submit', async ({ page }) => {
        const emailField = page.getByRole('textbox', { name: 'Email' });
        const passwordField = page.getByRole('textbox', { name: 'Password' });
        const submitButton = page.getByRole('button', { name: 'Log in' });

        await emailField.fill('admin@test.com');
        await passwordField.fill('123456');

        const loginRequestPromise = page.waitForResponse(basePath + '/login');
        await submitButton.click();
        const loginRequestResponse = await loginRequestPromise;

        expect(loginRequestResponse.status()).toBe(200);
    });
});