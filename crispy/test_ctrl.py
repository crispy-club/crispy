from crispy.ctrl import CCEvent, ccp
from crispy.filters import name


def test_plugin_cc_pattern_json() -> None:
    pattern = ccp([CCEvent(cc=102, value=1.0 / n) for n in range(1, 17)]) | name("foo")
    assert (
        pattern.json()
        == """{"name":"foo","events":[{"action":{"CtrlEvent":{"cc":102,"value":1.0}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.5}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.3333333333333333}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.25}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.2}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.16666666666666666}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.14285714285714285}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.125}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.1111111111111111}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.1}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.09090909090909091}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.08333333333333333}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.07692307692307693}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.07142857142857142}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.06666666666666667}},"dur":{"num":1,"den":16}},{"action":{"CtrlEvent":{"cc":102,"value":0.0625}},"dur":{"num":1,"den":16}}],"length_bars":{"num":1,"den":1}}"""
    )
