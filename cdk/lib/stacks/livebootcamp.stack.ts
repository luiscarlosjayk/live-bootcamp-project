import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as cloudfront from 'aws-cdk-lib/aws-cloudfront';
import * as cloudfrontOrigins from 'aws-cdk-lib/aws-cloudfront-origins';
import * as route53 from 'aws-cdk-lib/aws-route53';
import * as route53Targets from 'aws-cdk-lib/aws-route53-targets';
import * as certificates from 'aws-cdk-lib/aws-certificatemanager';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as logs from 'aws-cdk-lib/aws-logs';
import path from 'path';

export interface LiveBootCampStackProps extends cdk.StackProps {};

const DOMAIN_NAME = 'luiscarlosjayk.com';
const SUB_DOMAIN = `livebootcamp.cdn`;
const FULL_DOMAIN = `${SUB_DOMAIN}.${DOMAIN_NAME}`;

export class LiveBootCampStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: LiveBootCampStackProps) {
    super(scope, id, props);

    /**
     * Route 53 & ACM
     */
    const hostedZone = route53.HostedZone.fromLookup(this, 'HostedZone', {
      domainName: DOMAIN_NAME,
    });

    const certificate = new certificates.Certificate(this, 'ACMCertificate', {
      domainName: FULL_DOMAIN,
      validation: certificates.CertificateValidation.fromDns(hostedZone),
    });

    /**
     * S3 & CloudFront
     */
    const s3Bucket = new s3.Bucket(this, 'S3Bucket', {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      accessControl: s3.BucketAccessControl.PRIVATE,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      publicReadAccess: false,
      objectOwnership: s3.ObjectOwnership.BUCKET_OWNER_ENFORCED,
    });
    const s3LogBucket = new s3.Bucket(this, 'LogLiveRustBootCampBucket', {
      objectOwnership: s3.ObjectOwnership.OBJECT_WRITER,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      publicReadAccess: false,
    });

    // Create a Origun Access Control List (OACL)
    const oac = new cloudfront.CfnOriginAccessControl(this, 'OACL', {
      originAccessControlConfig: {
        name: 'LiveRustBootCampOACL',
        originAccessControlOriginType: 's3',
        signingBehavior: 'always',
        signingProtocol: 'sigv4',
      },
    });

    /**
     * CloudFront
     */
    const lambdaEdgeFn = new cloudfront.experimental.EdgeFunction(this, 'LambdaEdge', {
      runtime: lambda.Runtime.NODEJS_20_X,
      handler: 'jwt-protection.handler',
      code: lambda.Code.fromAsset(path.join(__dirname, '../../src/lambda')),
    });
    const lambdaEdgeFnVersion = lambdaEdgeFn.currentVersion;

    const cloudfrontDistribution = new cloudfront.Distribution(this, 'Distribution', {
      priceClass: cloudfront.PriceClass.PRICE_CLASS_100, // I'm not rich 💰 yet
      certificate: certificate,
      domainNames: [
        FULL_DOMAIN
      ],
      logBucket: s3LogBucket,
      enableLogging: true,
      defaultBehavior: {
        origin: new cloudfrontOrigins.S3Origin(s3Bucket),
        viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
        allowedMethods: cloudfront.AllowedMethods.ALLOW_GET_HEAD_OPTIONS,
        cachedMethods: cloudfront.CachedMethods.CACHE_GET_HEAD,
        compress: true,
        originRequestPolicy: cloudfront.OriginRequestPolicy.ALL_VIEWER_EXCEPT_HOST_HEADER,
        edgeLambdas: [
          {
            functionVersion: lambdaEdgeFnVersion,
            eventType: cloudfront.LambdaEdgeEventType.VIEWER_REQUEST,
            includeBody: true,
          }
        ],
      }
    });

    // Configure bucket policy to allow access from CloudFront
    s3Bucket.addToResourcePolicy(new iam.PolicyStatement({
      actions: ['s3:GetObject'],
      effect: iam.Effect.ALLOW,
      resources: [s3Bucket.arnForObjects('*')],
      principals: [
        new iam.ServicePrincipal('cloudfront.amazonaws.com'),
      ],
      conditions: {
        StringEquals: {
          'AWS:SourceArn': `arn:aws:cloudfront::${this.account}:distribution/${cloudfrontDistribution.distributionId}`,
        },
      },
    }));

    new route53.ARecord(this, 'Route53ARecord', {
      recordName: SUB_DOMAIN,
      zone: hostedZone,
      target: route53.RecordTarget.fromAlias(new route53Targets.CloudFrontTarget(cloudfrontDistribution)),
    });

  }
}
