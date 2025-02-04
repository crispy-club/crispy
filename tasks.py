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
def mypy_crispy(ctx):
    cmd = f"mypy --config-file {MYPY_CONFIG} {os.path.join(DIRNAME, 'crispy')}"
    print(cmd)
    ctx.run(cmd)


@task
def pytest(ctx):
    cmd = f"pytest --color=yes --junitxml=pytest.xml --cov-report=term-missing:skip-covered --cov=crispy {DIRNAME}"
    print(cmd)
    ctx.run(cmd)


@task(
    code_style,
    linters,
    mypy_crispy,
    pytest,
)
def build(_):
    pass
