from lark import Token, Tree

from livecoding.grammar import get_pattern_parser, get_transformer


def test_pattern_with_just_notes() -> None:
    parser = get_pattern_parser()
    tree = parser.parse("[c1 d#1 g1 c2]")
    assert get_transformer().transform(tree) == Tree[int](
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
    tree = parser.parse("[c1,1.0 d#1,0.8 g1,0.5 c2,0.9]")
    assert get_transformer().transform(tree) == Tree[tuple[int, float]](
        Token("RULE", "pattern"),
        [
            Tree[tuple[int, float]](Token("RULE", "event"), [(36, 1.0)]),
            Tree[tuple[int, float]](Token("RULE", "event"), [(39, 0.8)]),
            Tree[tuple[int, float]](Token("RULE", "event"), [(43, 0.5)]),
            Tree[tuple[int, float]](Token("RULE", "event"), [(48, 0.9)]),
        ],
    )


def test_nested_pattern_with_just_notes() -> None:
    parser = get_pattern_parser()
    tree = parser.parse("[c1 [c1 d#1 d1 d#1] g1 c2]")
    assert get_transformer().transform(tree) == Tree[int](
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
    tree = parser.parse("[c1 [c1,0.8 d#1,0.6 d1,0.9 d#1] g1,0.5 c2,0.55]")
    assert get_transformer().transform(tree) == Tree(
        Token("RULE", "pattern"),
        [
            Tree(Token("RULE", "event"), [36]),
            Tree(
                Token("RULE", "event"),
                [
                    Tree(
                        Token("RULE", "pattern"),
                        [
                            Tree(Token("RULE", "event"), [(36, 0.8)]),
                            Tree(Token("RULE", "event"), [(39, 0.6)]),
                            Tree(Token("RULE", "event"), [(38, 0.9)]),
                            Tree(Token("RULE", "event"), [39]),
                        ],
                    )
                ],
            ),
            Tree(Token("RULE", "event"), [(43, 0.5)]),
            Tree(Token("RULE", "event"), [(48, 0.55)]),
        ],
    )


def test_nested_pattern_with_notes_and_velocities_and_newlines() -> None:
    parser = get_pattern_parser()
    tree = parser.parse(
        """
    [
      c1
      [c1,0.8 d#1,0.75 d1,0.85 d#1]
      g1,0.4
      c2,0.6
    ]"""
    )
    assert get_transformer().transform(tree) == Tree(
        Token("RULE", "pattern"),
        [
            Tree(Token("RULE", "event"), [36]),
            Tree(
                Token("RULE", "event"),
                [
                    Tree(
                        Token("RULE", "pattern"),
                        [
                            Tree(Token("RULE", "event"), [(36, 0.8)]),
                            Tree(Token("RULE", "event"), [(39, 0.75)]),
                            Tree(Token("RULE", "event"), [(38, 0.85)]),
                            Tree(Token("RULE", "event"), [39]),
                        ],
                    )
                ],
            ),
            Tree(Token("RULE", "event"), [(43, 0.4)]),
            Tree(Token("RULE", "event"), [(48, 0.6)]),
        ],
    )
