import typing
from enum import Enum
from json import loads
from pathlib import Path
from requests import Session

DEFAULT_IDS_JSON_URL = (
    'https://raw.githubusercontent.com/blockworks-foundation/mango-explorer/main/data/ids.json'
)


class MarketType(Enum):
    PERP = 'perpMarkets'
    SPOT = 'spotMarkets'


def load_mango_markets_setup(
    ids_json: typing.Union[str, Path] = DEFAULT_IDS_JSON_URL
) -> typing.Dict:
    json_data: typing.Optional[typing.Dict] = None
    if isinstance(ids_json, str):
        if '://' in ids_json:  # expecting the thing is a downloadable url
            session: Session
            with Session() as session:
                response = session.get(ids_json)
                if response.status_code != 200:
                    raise Exception(f'Cannot get data from url {ids_json}, response: {response}')
                json_data = loads(response.text)
        else:
            ids_json = Path(ids_json)  # not an url, expecting paht
    if not json_data:
        json_string = ids_json.read_text()
        json_data = loads(json_string)
    if not json_data:
        raise Exception(f'Cannot get configuration from {ids_json}')
    return json_data


def get_group(
    group_name: str, ids_json: typing.Union[str, Path] = DEFAULT_IDS_JSON_URL
) -> typing.Dict:
    parsed_json = load_mango_markets_setup(ids_json)
    groups = parsed_json['groups']
    for group in groups:
        if group['cluster'] == group_name:  # TODO: this could be maybe name instead of 'cluster'
            return group
    raise Exception(f'Group {group_name} does not exist in ids config at {ids_json}')


def get_market(
    group_name: str,
    market_type: MarketType,
    ids_json: typing.Union[str, Path] = DEFAULT_IDS_JSON_URL,
) -> typing.Dict:
    parsed_group = get_group(group_name, ids_json)
    market_json = parsed_group[market_type.value]
    if market_json:
        return market_json
    raise Exception(f'Market {market_type} for group {group_name} does not exist at {ids_json}')


def get_symbol(
    group_name: str,
    symbol_name: str,
    ids_json: typing.Union[str, Path] = DEFAULT_IDS_JSON_URL,
) -> typing.Dict:
    parsed_group = get_group(group_name, ids_json)
    for market_type in MarketType:
        if not parsed_group[market_type.value]:
            Exception(
                f'Market type {market_type.value} not defined for group {group_name} at {ids_json}'
            )
        for market in parsed_group[market_type.value]:
            if market['name'] == symbol_name:
                return {
                    'cluster': parsed_group['cluster'],
                    'name': parsed_group['name'],
                    'publicKey': parsed_group['publicKey'],  # group pubkey
                    'quoteSymbol': parsed_group['quoteSymbol'],
                    'mangoProgramId': parsed_group['mangoProgramId'],
                    'serumProgramId': parsed_group['serumProgramId'],
                    'market_type': market_type.value,
                    'market': market,
                }
    raise Exception(f'Market {market_type} for group {group_name} does not exist at {ids_json}')
