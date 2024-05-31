import type { Context, CloudFrontFunctionsEvent, CloudFrontResultResponse } from 'aws-lambda';
import * as jwt from 'jsonwebtoken';

type HandlerResponse = CloudFrontFunctionsEvent['request'] | CloudFrontFunctionsEvent['response'];

const JWT_SECRET = process.env?.JWT_SECRET;
const JWT_HEADER_NAME = process.env?.JWT_HEADER_NAME ?? 'jwt';

export async function handler(event: CloudFrontFunctionsEvent, _context: Context): Promise<HandlerResponse> {
    const request = event.request;
    const cookies = request.cookies;

    // Verify JWT_SECRET is set
    if (typeof JWT_SECRET !== 'string' || JWT_SECRET.length === 0) {
        console.error('JWT_SECRET is undefined or empty');
        return generateInternalServerErrorResponse();
    }

    // Verify JWT_HEADER_NAME is set
    if (typeof JWT_HEADER_NAME !== 'string' || JWT_HEADER_NAME.length === 0) {
        console.error('JWT_HEADER_NAME is undefined or empty');
        return generateInternalServerErrorResponse();
    }

    // Verify the request is a GET request
    if (request.method !== 'GET') {
        return generateErrorResponse();
    }

    // // Verify the request is for the protected resource
    // if (request.uri! == '/protected') {
    //     return generateErrorResponse();
    // }

    try {
        // Verify jwt cookie
        if (!cookies || (JWT_HEADER_NAME in cookies)) {
            return generateErrorResponse();
        }

        const jwtCookie = cookies[JWT_HEADER_NAME];
        const token = jwtCookie.value;

        if (!token) {
            return generateErrorResponse();
        }

        // Verify the token using the secret
        jwt.verify(token, JWT_SECRET);

        // Token is valid, allow the request to proceed
        return request;
    } catch (err) {
        // Check if error is due to invalid token
        if (err instanceof jwt.JsonWebTokenError) {
            return generateErrorResponse();
        }
        // For other types of errors, return a 500 response
        return generateInternalServerErrorResponse();
    }
}

function generateErrorResponse(): CloudFrontFunctionsEvent['response'] {
    return {
        statusCode: 401,
        statusDescription: 'Unauthorized',
        headers: {
            'www-authenticate': {
                value: 'Bearer realm="LiveBootcampAPI", error="invalid_token", error_description="The id token is invalid or expired.'
            },
        },
        cookies: {},
    };
}

function generateInternalServerErrorResponse(): CloudFrontFunctionsEvent['response'] {
    return {
        statusCode: 500,
        statusDescription: 'Internal Server Error',
        headers: {
            'content-type': {
                value: 'plain/text',
            },
        },
        cookies: {},
    };
}
