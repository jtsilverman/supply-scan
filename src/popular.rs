pub fn npm_packages() -> &'static [&'static str] {
    &[
        "express", "react", "lodash", "axios", "chalk", "moment", "typescript", "webpack",
        "babel", "jest", "eslint", "prettier", "next", "vue", "angular", "jquery", "underscore",
        "async", "debug", "commander", "yargs", "inquirer", "ora", "glob", "minimatch", "semver",
        "uuid", "dotenv", "cors", "body-parser", "cookie-parser", "jsonwebtoken", "bcrypt",
        "mongoose", "sequelize", "prisma", "pg", "mysql2", "redis", "socket.io", "ws",
        "http-proxy", "node-fetch", "got", "superagent", "cheerio", "puppeteer", "playwright",
        "sharp", "jimp", "multer", "formidable", "passport", "helmet", "morgan", "winston",
        "pino", "bunyan", "nodemon", "pm2", "concurrently", "cross-env", "rimraf", "mkdirp",
        "fs-extra", "path", "crypto", "zod", "joi", "yup", "ajv", "luxon", "dayjs", "date-fns",
        "ramda", "rxjs", "immer", "mobx", "redux", "zustand", "swr", "react-query",
        "styled-components", "tailwindcss", "postcss", "sass", "less", "esbuild", "vite",
        "rollup", "parcel", "turbopack", "fastify", "koa", "hapi", "restify", "graphql",
        "apollo-server",
    ]
}

pub fn pypi_packages() -> &'static [&'static str] {
    &[
        "requests", "flask", "django", "numpy", "pandas", "scipy", "matplotlib", "scikit-learn",
        "tensorflow", "pytorch", "torch", "transformers", "pillow", "opencv-python",
        "beautifulsoup4", "scrapy", "selenium", "pytest", "unittest2", "tox", "black", "flake8",
        "mypy", "pylint", "isort", "setuptools", "wheel", "pip", "twine", "virtualenv", "poetry",
        "pipenv", "pyyaml", "toml", "tomli", "jsonschema", "pydantic", "fastapi", "uvicorn",
        "gunicorn", "celery", "redis", "boto3", "botocore", "awscli", "google-cloud-storage",
        "azure-storage-blob", "sqlalchemy", "psycopg2", "pymongo", "motor", "aiohttp", "httpx",
        "urllib3", "certifi", "charset-normalizer", "idna", "six", "packaging",
        "typing-extensions", "click", "rich", "typer", "tqdm", "loguru", "python-dotenv",
        "jinja2", "mako", "markupsafe", "cryptography", "paramiko", "fabric", "ansible", "docker",
        "kubernetes", "pytest-cov", "coverage", "hypothesis", "faker", "factory-boy",
        "marshmallow", "attrs", "cattrs", "dataclasses-json", "orjson", "ujson", "msgpack",
        "protobuf", "grpcio", "websockets", "starlette", "sanic", "tornado", "twisted", "gevent",
        "eventlet", "networkx", "sympy", "nltk", "spacy", "gensim", "huggingface-hub", "datasets",
        "tokenizers", "accelerate", "diffusers", "langchain", "openai", "anthropic",
    ]
}
