from wonderwords import RandomWord, Defaults


def random_name() -> str:
    adj, noun = RandomWord(adj=Defaults.ADJECTIVES), RandomWord(adj=Defaults.NOUNS)
    return f"{adj.word()}-{noun.word()}"
