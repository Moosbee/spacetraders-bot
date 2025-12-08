import type { CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
  overwrite: true,
  schema: "http://127.0.0.1:8780/",
  documents: "src/**/*.{ts,tsx}",
  watch: true,

  generates: {
    "src/gql/": {
      preset: "client",
      plugins: [],
      config: {
        scalars: {
          DateTime: "string",
        },
      },
    },
  },
};

export default config;
