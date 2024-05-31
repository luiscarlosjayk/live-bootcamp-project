#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { LiveBootCampStack } from '../lib/stacks/livebootcamp.stack';

const AWS_ACCOUNT = process.env.AWS_ACCOUNT || process.env.CDK_DEFAULT_ACCOUNT;
const AWS_REGION = process.env.AWS_REGION || process.env.CDK_DEFAULT_REGION;

if (!AWS_ACCOUNT || !AWS_REGION) {
  throw new Error('AWS_ACCOUNT and AWS_REGION must be set');
}

const app = new cdk.App();
new LiveBootCampStack(app, 'LiveBootCampStack', {
  stackName: 'livebootcamp-stack',
  env: {
    account: AWS_ACCOUNT,
    region: AWS_REGION,
  },
  tags: {
    OWNER: 'Luis Carlos Osorio Jayk',
    team: 'LiveBootCamp',
    application: 'LiveBootCamp',
  },
});