import { test, expect, type Page } from '@playwright/test';
import data from "../auth-service/postman/collections/data.json";

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

test.describe('Signup', () => {
    data.forEach((request, index) => {
        test(`Iteration:${index} - Should return ${request.expected}`, async ({ page }) => {
            const signUpLink = page.getByRole('link', { name: 'Sign up here' });
            await signUpLink.waitFor();
            await signUpLink.click();
            
            const emailField = page.getByRole('textbox', { name: 'Email' });
            const passwordField = page.getByRole('textbox', { name: 'Password' });
            const twoFACheckbox = page.locator('#signup-2FA-checkbox');
            const submitButton = page.getByRole('button', { name: 'Sign up' });

            await emailField.fill(request.body.email);
            await passwordField.fill(request.body.password);
            if (request.body.requires2FA) {
                await twoFACheckbox.check();
            }

            const requestPromise = page.waitForResponse(basePath + '/signup');
            await submitButton.click();
            const requestResponse = await requestPromise;

            expect(requestResponse.status()).toBe(request.expected);
        });
    });
});