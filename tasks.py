import os

from invoke import task

DIRNAME = os.path.dirname(__file__)
MYPY_CONFIG = os.path.join(DIRNAME, "mypy.ini")


@task
def linters(ctx):
    cmd = "ruff check"
    print(cmd)
    ctx.run(cmd)


@task
def code_style(ctx):
    cmd = "ruff format --check"
    print(cmd)
    ctx.run(cmd)


@task
def mypy_livecoding(ctx):
    cmd = f"mypy --config-file {MYPY_CONFIG} {os.path.join(DIRNAME, 'livecoding')}"
    print(cmd)
    ctx.run(cmd)


@task
def pytest(ctx):
    cmd = f"pytest --color=yes {DIRNAME}"
    print(cmd)
    ctx.run(cmd)


@task
def cargo_test(ctx):
    cmd = "cargo test --package livecoding"
    print(cmd)
    ctx.run(cmd)


@task(
    code_style,
    linters,
    mypy_livecoding,
    pytest,
    cargo_test,
)
def build(_):
    pass
