import * as esbuild from 'esbuild';

await esbuild.build({
    entryPoints: ['./src/lambda/**.ts'],
    outdir: './src/lambda',
    bundle: true,
    sourcemap: 'inline',
    platform: 'node',
    target: 'esnext',
    define: {
        'process.env.JWT_SECRET': `'${process.env.JWT_SECRET}'`,
    },
});
