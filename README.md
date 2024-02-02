# Benjamin Assistant

**Benjamin** is a personal assistant that can help you build a web server in any language by only providing the template you want to use.
Primary supported backend programming language: **Rust**

## Setup

Create a `.env` file in the root of the project and add the following:

```env
OPEN_AI_ORG=<YOUR_ORG>
OPEN_AI_KEY=<YOUR_KEY>
GPT_MODEL=<YOUR_MODEL> # gpt-3.5-turbo, gpt-4
```

Moreover, add these to .env file if you want to use the server:

- The absolute path to the template you want to use.
```env
CODE_EXECUTE_TEMPLATE_ABSOLUTE_PATH=<your-template-path>
```

- The absolute path to the main file of the project you want to create.
```env
EXEC_MAIN_ABSOLUTE_PATH=<your-main-file-path>
```

- The absolute path to the api endpoints schema file.
```env
API_SCHEMA_ABSOLUTE_PATH=<your-api-schema-path>
```

- Tthe absolute path to the web server project. It is the same as the EXEC_MAIN_ABSOLUTE_PATH.
```env
WEB_SERVER_PROJECT_ABSOLUTE_PATH=<your-web-server-project-path>
```

**Note**: You must create the directories and files before running the server.

## Usage

To use the server, run the following command:

```bash
cargo build --release
./target/release/benjamin-assistant
```

or if you want to use the CLI, run the following command:

```bash
cargo run
```

Then you can start using the assistant by answering the prompts.


# Limitations
- The assistant has not been tested for building a frontend project yet.
- The prompts only allow the model to generate responses according to the given code template.
