# News Aggregator

## General Information
News Aggregator is a project which aggregates the news from the RSS feeds of multiple news media. It allows the user to view the latest news available in the RSS feeds of the supported news media. User can select the amount of news to get from each source by specifying it in the request, or use the default request to get the constant 3 articles per source. Current list of media includes:
- BBC News
- POLITICO
- The Guardian

## Technical details
Current implementation has the next specifics:
- Response provided to the end user is in the JSON format.
- List of sources is hardcoded and to extend it the source code has to be updated.
- All users can get news from the same list of sources.
- Only RSS XML feeds are supported because they have the same structure and are easy to implement.
- The search through the APIs of the supported sources are not used because it usually requires obtaining a developer or other type of API key. Also, each API is very specific and requires a separate implementation to be supported.
- The separate UI is not implemented. For now, end users can use the Candid generated one or direct API calls.

## How to access & use
To access the Candid UI and perform the call to canister use this url: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=zxhp2-oyaaa-aaaam-qdjoa-cai <br>
To check the information about canister use this url: https://dashboard.internetcomputer.org/canister/zxhp2-oyaaa-aaaam-qdjoa-cai <br>
The UI has 2 available methods:
- ``get_aggregated_news`` - returns the list containing 3 news for each source. It is used if user does not want to specify the amount of news per source directly. You can think of it as a default value of news per source.
- ``get_aggregated_news_limited`` with ``nat8`` parameter - returns the list containing the amount of news for each source provided by the user using the ``nat8`` parameter. If provided number is 0 - user will get no news and error notifying that the ``nat8`` parameter must be greater than 0.