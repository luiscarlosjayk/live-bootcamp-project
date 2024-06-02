import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as cloudfront from 'aws-cdk-lib/aws-cloudfront';
import * as route53 from 'aws-cdk-lib/aws-route53';
import * as route53Targets from 'aws-cdk-lib/aws-route53-targets';
import * as certificates from 'aws-cdk-lib/aws-certificatemanager';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import path from 'path';
import { S3OACOrigin } from '../constructs/s3-oac-origin';

export interface LiveBootCampStackProps extends cdk.StackProps {
  domainName: string;
  subDomain: string;
};

export class LiveBootCampStack extends cdk.Stack {
  constructor(scope: Construct, id: string, { domainName, subDomain, ...props }: LiveBootCampStackProps) {
    super(scope, id, props);

    const fullDomainName = `${subDomain}.${domainName}`;

    /**
     * Route 53 & ACM
     */
    const hostedZone = route53.HostedZone.fromLookup(this, 'HostedZone', {
      domainName: domainName,
    });

    /**
     * SSL certificate to enable secure traffic through HTTPS only
     */
    const certificate = new certificates.Certificate(this, 'ACMCertificate', {
      domainName: fullDomainName,
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

    /**
     * Lambda@Edge function
     */
    const lambdaEdgeFn = new cloudfront.experimental.EdgeFunction(this, 'LambdaEdge', {
      runtime: lambda.Runtime.NODEJS_20_X,
      handler: 'jwt-protection.handler',
      code: lambda.Code.fromAsset(path.join(__dirname, '../../src/lambda')),
    });
    const lambdaEdgeFnVersion = lambdaEdgeFn.currentVersion;

    /**
     * CloudFront
     */
    const oacOrigin = new S3OACOrigin(s3Bucket);
    const cloudfrontDistribution = new cloudfront.Distribution(this, 'Distribution', {
      priceClass: cloudfront.PriceClass.PRICE_CLASS_100, // I'm not rich 💰 yet
      certificate: certificate,
      domainNames: [
        fullDomainName
      ],
      logBucket: s3LogBucket,
      enableLogging: true,
      defaultBehavior: {
        origin: oacOrigin,
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

    // This is needed to add the rules that will allow the distribution read content from private bucket with OAC
    oacOrigin.addResourcePolicy(cloudfrontDistribution);

    // Link subdomain to the distribution in the existing HostedZone
    new route53.ARecord(this, 'Route53ARecord', {
      recordName: subDomain,
      zone: hostedZone,
      target: route53.RecordTarget.fromAlias(new route53Targets.CloudFrontTarget(cloudfrontDistribution)),
    });
  }
}
