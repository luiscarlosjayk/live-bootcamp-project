import type { Context, CloudFrontFunctionsEvent } from 'aws-lambda';

type HandlerResponse = CloudFrontFunctionsEvent['request'] | CloudFrontFunctionsEvent['response'];
const JWT_SECRET = process.env.JWT_SECRET;

export async function handler(event: CloudFrontFunctionsEvent, _context: Context): Promise<HandlerResponse> {
    console.log(event);
    console.log(typeof JWT_SECRET);
    console.log(JWT_SECRET?.length);
    return event.request;
}
