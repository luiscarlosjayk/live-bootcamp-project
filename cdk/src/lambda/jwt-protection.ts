import type { Context, CloudFrontResponseEvent, Callback } from 'aws-lambda';
import * as jwt from 'jsonwebtoken';

const JWT_SECRET = process.env.JWT_SECRET;

function verifyToken(token: string, secret: string) {
    return jwt.verify(token, secret);
}

export function handler(event: CloudFrontResponseEvent, _context: Context, callback: Callback) {
    const request = event.Records[0].cf.request;

    console.log(JSON.stringify(request, null, 2));

    if (typeof JWT_SECRET !== 'string') {
        console.log('Missing JWT_SECRET env');
        console.info(response500);
        return callback(null, response500);
    }

    // if (request.uri !== '/certificate.png') {
    //     callback(null, notFoundErrorResponse());
    //     return;
    // }

    const querystring = request.querystring;
    const queryParams = <{ [key: string]: string; }>{};

    if (querystring) {
        querystring.split('&').forEach(param => {
            const [key, value] = param.split('=');
            queryParams[key] = value;
        });
    }
    const token = queryParams['token'];
    console.debug(`token: ${token}`);

    if (!token) {
        console.error('Missing token param');
        console.info(response500);
        return callback(null, response500);
    }

    try {
        const decoded = verifyToken(token, JWT_SECRET);
        print(decoded);
    } catch (error) {
        console.error(`Invalid token: ${token}`);
        console.info(response401);
        return callback(null, response401);
    }

    console.info('Success');
    callback(null, request); // Let it pass 👑
}

function print(data: string | Record<string, any>): void {
    if (typeof data === 'string') {
        console.log(data);
    } else {
        console.log(JSON.stringify({ data }));
    }
}

const response401 = {
    status: "401",
    statusDescription: "Unauthorized",
};

const response404 = {
    status: "404",
    statusDescription: "Not Found",
};

const response500 = {
    status: 500,
    statusDescription: 'Internal Server Error',
};
