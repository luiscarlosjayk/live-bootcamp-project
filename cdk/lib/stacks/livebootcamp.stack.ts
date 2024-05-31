import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as cloudfront from 'aws-cdk-lib/aws-cloudfront';
import * as cloudfrontOrigins from 'aws-cdk-lib/aws-cloudfront-origins';
import * as route53 from 'aws-cdk-lib/aws-route53';
import * as route53Targets from 'aws-cdk-lib/aws-route53-targets';
import * as certificates from 'aws-cdk-lib/aws-certificatemanager';
import * as iam from 'aws-cdk-lib/aws-iam';

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
    // const hostedZone = route53.HostedZone.fromLookup(this, 'HostedZone', {
    //   domainName: DOMAIN_NAME,
    // });

    // const certificate = new certificates.Certificate(this, 'LiveBootCampCertificate', {
    //   domainName: FULL_DOMAIN,
    //   validation: certificates.CertificateValidation.fromDns(hostedZone),
    // });

    /**
     * S3 & CloudFront
     */
    const s3Bucket = new s3.Bucket(this, 'LiveBootCampBucket', {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: true,
      // accessControl: s3.BucketAccessControl.PRIVATE,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      publicReadAccess: false,
    });
    const s3LogBucket = new s3.Bucket(this, 'LiveBootCampLogBucket', {
      objectOwnership: s3.ObjectOwnership.OBJECT_WRITER,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: true,
      // accessControl: s3.BucketAccessControl.PRIVATE,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      publicReadAccess: false,
    });

    /**
     * CloudFront
     */
    const jwtValidatorFunction = new cloudfront.Function(this, 'LiveBootCampJWTValidator', {
      code: cloudfront.FunctionCode.fromFile({
        // filePath: 'src/lambda/jwt-validator.js',
        filePath: './src/lambda/dummy.js',
      }),
      runtime: cloudfront.FunctionRuntime.JS_2_0,
      autoPublish: true, // Automatically publish the function to the LIVE stage when it’s created.
    });

    const cloudfrontDistribution = new cloudfront.Distribution(this, 'LiveBootCampDistribution', {
      priceClass: cloudfront.PriceClass.PRICE_CLASS_100, // I'm not rich 💰 yet
      // certificate: certificate,
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
        originRequestPolicy: new cloudfront.OriginRequestPolicy(this, 'OriginRequestPolicy', {
          cookieBehavior: cloudfront.OriginRequestCookieBehavior.all(),
          headerBehavior: cloudfront.OriginRequestHeaderBehavior.all(),
          queryStringBehavior: cloudfront.OriginRequestQueryStringBehavior.all()
        }),
        functionAssociations: [{
          function: jwtValidatorFunction,
          eventType: cloudfront.FunctionEventType.VIEWER_REQUEST,
        }],
      }
    });

    // Configure bucket policy to allow access from CloudFront
    s3Bucket.addToResourcePolicy(new iam.PolicyStatement({
      actions: ['s3:GetObject'],
      effect: iam.Effect.ALLOW,
      resources: [s3Bucket.arnForObjects('*')],
      // principals: [new iam.CanonicalUserPrincipal(cloudfrontDistribution.distributionDomainName)],
      principals: [
        new iam.ServicePrincipal('cloudfront.amazonaws.com'),
      ],
      conditions: {
        StringEquals: {
          'AWS:SourceArn': `arn:aws:cloudfront::${this.account}:distribution/${cloudfrontDistribution.distributionId}`
        }
      },
    }));


    // new route53.ARecord(this, 'LiveBootCampARecord', {
    //   recordName: FULL_DOMAIN,
    //   zone: hostedZone,
    //   target: route53.RecordTarget.fromAlias(new route53Targets.CloudFrontTarget(cloudfrontDistribution)),
    // });

    /**
     * Stack Outputs
     */
    new cdk.CfnOutput(this, 'LiveBootCampURL', {
      value: cloudfrontDistribution.distributionDomainName,
    });

    new cdk.CfnOutput(this, 'S3 Bucket', {
      value: s3Bucket.bucketName,
    });

    new cdk.CfnOutput(this, 'JWT Validator Function', {
      value: jwtValidatorFunction.functionName,
    });

  }
}
