from lark import Token, Tree

from livecoding.grammar import get_pattern_parser
from livecoding.pattern import _get_transformer


def test_pattern_with_just_notes() -> None:
    parser = get_pattern_parser()
    tree = parser.parse("[c1 d#1 g1 c2]")
    assert _get_transformer().transform(tree) == Tree[int](
        Token("RULE", "pattern"),
        [
            Tree[int](Token("RULE", "event"), [36]),
            Tree[int](Token("RULE", "event"), [39]),
            Tree[int](Token("RULE", "event"), [43]),
            Tree[int](Token("RULE", "event"), [48]),
        ],
    )


def test_pattern_with_notes_and_velocities() -> None:
    parser = get_pattern_parser()
    tree = parser.parse("[c1,127 d#1,60 g1,50 c2,110]")
    assert _get_transformer().transform(tree) == Tree[tuple[int, int]](
        Token("RULE", "pattern"),
        [
            Tree[tuple[int, int]](Token("RULE", "event"), [(36, 127)]),
            Tree[tuple[int, int]](Token("RULE", "event"), [(39, 60)]),
            Tree[tuple[int, int]](Token("RULE", "event"), [(43, 50)]),
            Tree[tuple[int, int]](Token("RULE", "event"), [(48, 110)]),
        ],
    )


def test_nested_pattern_with_just_notes() -> None:
    parser = get_pattern_parser()
    tree = parser.parse("[c1 [c1 d#1 d1 d#1] g1 c2]")
    assert _get_transformer().transform(tree) == Tree[int](
        Token("RULE", "pattern"),
        [
            Tree(Token("RULE", "event"), [36]),
            Tree(
                Token("RULE", "event"),
                [
                    Tree(
                        Token("RULE", "pattern"),
                        [
                            Tree(Token("RULE", "event"), [36]),
                            Tree(Token("RULE", "event"), [39]),
                            Tree(Token("RULE", "event"), [38]),
                            Tree(Token("RULE", "event"), [39]),
                        ],
                    )
                ],
            ),
            Tree(Token("RULE", "event"), [43]),
            Tree(Token("RULE", "event"), [48]),
        ],
    )


def test_nested_pattern_with_notes_and_velocities() -> None:
    parser = get_pattern_parser()
    tree = parser.parse("[c1 [c1,100 d#1,80 d1,90 d#1] g1,50 c2,70]")
    print(_get_transformer().transform(tree))
    assert _get_transformer().transform(tree) == Tree(
        Token("RULE", "pattern"),
        [
            Tree(Token("RULE", "event"), [36]),
            Tree(
                Token("RULE", "event"),
                [
                    Tree(
                        Token("RULE", "pattern"),
                        [
                            Tree(Token("RULE", "event"), [(36, 100)]),
                            Tree(Token("RULE", "event"), [(39, 80)]),
                            Tree(Token("RULE", "event"), [(38, 90)]),
                            Tree(Token("RULE", "event"), [39]),
                        ],
                    )
                ],
            ),
            Tree(Token("RULE", "event"), [(43, 50)]),
            Tree(Token("RULE", "event"), [(48, 70)]),
        ],
    )
