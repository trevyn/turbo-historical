import pkg from "@graphql-codegen/cli";
const { generate } = pkg;

let schema = {
 "http://localhost:3020/graphql": {
  headers: { Authorization: "Bearer foo" },
 },
};

async function run() {
 console.log("generating graphql.schema.json...");
 await generate(
  {
   schema: schema,
   overwrite: true,
   generates: {
    "./graphql.schema.json": {
     plugins: ["introspection"],
    },
   },
  },
  true
 );

 console.log("generating generated-queries.graphql...");
 await generate(
  {
   schema: schema,
   overwrite: true,
   generates: {
    "./generated-queries.graphql": {
     config: {
      schemaJsonFile: "./graphql.schema.json",
     },
     plugins: ["queries-plugin.js"],
    },
   },
  },
  true
 );

 console.log("generating src/graphql-codegen.ts...");
 await generate(
  {
   schema: schema,
   overwrite: true,
   generates: {
    "./src/graphql-codegen.ts": {
     documents: "./generated-queries.graphql",
     plugins: ["typescript", "typescript-operations", "typescript-react-apollo"],
     config: {
      skipTypename: false,
      withHooks: true,
      withHOC: false,
      withComponent: false,
      apolloReactHooksImportFrom: "@apollo/client",
      scalars: { i53: "number" },
     },
    },
   },
  },
  true
 );
}

run();
