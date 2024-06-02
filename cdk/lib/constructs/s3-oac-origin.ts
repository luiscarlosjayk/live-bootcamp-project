import { Construct } from 'constructs';
import * as cloudfront from 'aws-cdk-lib/aws-cloudfront';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as cdk from 'aws-cdk-lib';

export interface S3OACOriginProps extends cloudfront.OriginProps {
    readonly originAccessControl?: cloudfront.CfnOriginAccessControl;
}

export interface S3OACOriginBindConfig extends cloudfront.OriginBindConfig {
    originProperty: cloudfront.OriginBindConfig['originProperty'] & {
        originAccessControlId?: string; //cloudfront.CfnDistribution.OriginProperty['originAccessControlId'];
    }
}

export class S3OACOrigin implements cloudfront.IOrigin {
    private readonly origin: cloudfront.IOrigin;
    private bucket: s3.IBucket;
    private originAccessControl: cloudfront.CfnOriginAccessControl;

    constructor(bucket: s3.IBucket, props: S3OACOriginProps = {}) {
        this.bucket = bucket;

        if (!props.originAccessControl) {
            const stack = cdk.Stack.of(bucket);
            const oacId = cdk.Names.uniqueId(stack);
            this.originAccessControl = new cloudfront.CfnOriginAccessControl(stack, oacId, {
                originAccessControlConfig: {
                  name: oacId,
                  originAccessControlOriginType: 's3',
                  signingBehavior: 'always',
                  signingProtocol: 'sigv4',
                },
            });
        }

        if (bucket.isWebsite) {
            throw 'S3OACOrigin does not work with bucket configured as website.';
        } else {
            this.origin = new S3BucketOACOrigin(bucket, props);
        }
    }

    bind(scope: Construct, options: cloudfront.OriginBindOptions) {
        const originalBindConfig = this.origin.bind(scope, options);
        
        if (originalBindConfig.originProperty) {
            const newBindConfig =  Object.assign(
                {},
                originalBindConfig,
                {
                    originProperty: {
                        ...originalBindConfig.originProperty,
                        originAccessControlId: this.originAccessControl.attrId
                    }
                }
            );

            return newBindConfig;
        }
        return originalBindConfig;
    }

    public addResourcePolicy(distribution: cloudfront.Distribution): void {
        const account = distribution.env.account;
        this.bucket.addToResourcePolicy(new iam.PolicyStatement({
            actions: ['s3:GetObject'],
            effect: iam.Effect.ALLOW,
            resources: [this.bucket.arnForObjects('*')],
            principals: [
              new iam.ServicePrincipal('cloudfront.amazonaws.com'),
            ],
            conditions: {
              StringEquals: {
                'AWS:SourceArn': `arn:aws:cloudfront::${account}:distribution/${distribution.distributionId}`,
              },
            },
        }));
    }
}

class S3BucketOACOrigin extends cloudfront.OriginBase {
    public originAccessControl: cloudfront.CfnOriginAccessControl;

    constructor(private readonly bucket: s3.IBucket, props: cloudfront.OriginProps) {
        super(bucket.bucketRegionalDomainName, props);
    }

    /**
     * As we're using origin access control (OAC) instead of access control identity (OAI)
     * we need to specify an empty originAccessIdentity property.
     * References:
     *  - https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-cloudfront-distribution-s3originconfig.html
     *  - https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-s3.html
     */
    protected renderS3OriginConfig(): cloudfront.CfnDistribution.S3OriginConfigProperty | undefined {
        return {
            originAccessIdentity: '',
        };
    }
}