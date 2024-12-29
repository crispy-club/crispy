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
    cmd = f"pytest --color=yes --junitxml=pytest.xml --cov-report=term-missing:skip-covered --cov=livecoding {DIRNAME}"
    print(cmd)
    ctx.run(cmd)


@task(
    code_style,
    linters,
    mypy_livecoding,
    pytest,
)
def build(_):
    pass
