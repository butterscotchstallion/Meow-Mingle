import {defineConfig} from '@hey-api/openapi-ts';

export default defineConfig({
    input: '../../openapi.json',
    output: {
        path: 'api',
        tsConfigPath: './tsconfig.json',
        case: 'camelCase',
    },
    plugins: [
        {
            name: '@hey-api/typescript',
            enums: 'typescript',
        },
        '@hey-api/sdk',
        {
            name: '@tanstack/react-query',
        },
    ],
});