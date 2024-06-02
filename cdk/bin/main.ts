#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { LiveBootCampStack } from '../lib/stacks/livebootcamp.stack';

const AWS_ACCOUNT = process.env.AWS_ACCOUNT || process.env.CDK_DEFAULT_ACCOUNT;
const AWS_REGION = process.env.AWS_REGION || process.env.CDK_DEFAULT_REGION;
const DOMAIN_NAME = process.env.DOMAIN_NAME;
const SUB_DOMAIN = process.env.SUB_DOMAIN;

if (!DOMAIN_NAME || !SUB_DOMAIN) {
  throw new Error('DOMAIN_NAME and SUB_DOMAIN must be set');
}

if (!AWS_ACCOUNT || !AWS_REGION) {
  throw new Error('AWS_ACCOUNT and AWS_REGION must be set');
}

const app = new cdk.App();
new LiveBootCampStack(app, 'LiveRustBootCampStack', {
  stackName: 'liverustbootcamp',
  domainName: DOMAIN_NAME,
  subDomain: SUB_DOMAIN,
  env: {
    account: AWS_ACCOUNT,
    region: AWS_REGION,
  },
  tags: {
    OWNER: 'LuisCarlosOsorioJayk',
    team: 'LiveBootCamp',
    application: 'LiveBootCamp',
  },
});

app.synth();
