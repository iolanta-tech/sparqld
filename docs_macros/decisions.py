"""Decision-log macro."""

from pydantic import BaseModel, ConfigDict

from docs_macros.adr import ADR_STATUS, human_date
from docs_macros.locations import QUERIES_DIR
from mkdocs_macros_sparqld import run_query
from mkdocs_macros_sparqld.render import SelectResult, Term


class Decision(BaseModel):
    """One row projected by the decision-log SELECT query."""

    model_config = ConfigDict(extra='ignore')

    graph: Term
    title: Term
    status: Term
    date: Term
    author: Term


Decision.model_rebuild(_types_namespace={'Term': Term})


def decisions(body: str) -> list[Decision]:
    selected = SelectResult.model_validate_json(body)
    return [Decision.model_validate(binding) for binding in selected.solutions.bindings]


def decision_heading(decision: Decision) -> str:
    title = decision.title.lexical_form
    page = decision.graph.lexical_form.rsplit('/', 1)[-1]
    return f'-   __[{title}]({page})__'


def decision_card(decision: Decision) -> list[str]:
    status_key = decision.status.lexical_form.lower()
    status = ADR_STATUS.get(
        status_key,
        status_key.replace('_', ' ').title(),
    )
    created = human_date(decision.date.lexical_form)
    return [
        decision_heading(decision),
        '',
        '    ---',
        '',
        f'    <span class="adr-status adr-status--{status_key}">{status}</span>',
        '',
        f'    :material-calendar-outline: <span class="adr-date">{created}</span>',
        '',
    ]


def decision_log() -> str:
    query = (QUERIES_DIR / 'decisions.rq').read_text()
    _, body = run_query(query)
    cards = ['<div class="grid cards adr-cards" markdown>', '']
    for decision in decisions(body):
        cards.extend(decision_card(decision))
    cards.append('</div>')
    return '\n'.join(cards)
