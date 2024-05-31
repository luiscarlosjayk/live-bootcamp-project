import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as cloudfront from 'aws-cdk-lib/aws-cloudfront';
import * as cloudfrontOrigins from 'aws-cdk-lib/aws-cloudfront-origins';

export interface LiveBootCampStackProps extends cdk.StackProps {}

export class LiveBootCampStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: LiveBootCampStackProps) {
    super(scope, id, props);

    const s3Bucket = new s3.Bucket(this, 'LiveBootCampBucket', {});
    
    const cloudfrontKeyValueStore = new cloudfront.KeyValueStore(this, 'LiveBootCampKeyValueStoreAsset', {
      source: cloudfront.ImportSource.fromInline(JSON.stringify({
        data: [
          {
            key: 'JWT_SECRET',
            value: process.env.JWT_SECRET,
          },
        ],
      })),
    });

    const jwtValidatorFunction = new cloudfront.Function(this, 'LiveBootCampJWTValidator', {
      code: cloudfront.FunctionCode.fromFile({
        // filePath: 'src/lambda/jwt-validator.js',
        filePath: 'src/lambda/dummy.js',
      }),
      runtime: cloudfront.FunctionRuntime.JS_2_0,
      autoPublish: true, // Automatically publish the function to the LIVE stage when it’s created.
      keyValueStore: cloudfrontKeyValueStore,
    });

    const cloudfrontDistribution = new cloudfront.Distribution(this, 'LiveBootCampDistribution', {
      priceClass: cloudfront.PriceClass.PRICE_CLASS_100, // I'm not rich 💰 yet
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
