# UGC data

UGC tf2 data for highlander, sixes, fours and ultiduo.

## teams.json

Basic data for each team

#### Sample

```json
[
  {
    "id": 7861,
    "tag": "-Xe-",
    "name": "Xenon",
    "image": "https://steamcdn-a.akamaihd.net/steamcommunity/public/images/avatars/db/dbabbd8bab7ccf6d27a9d4ca2e73a76e085bb201_full.jpg",
    "format": "9v9",
    "region": "europe",
    "timezone": "West-Euro"
  }
]
```

## players.json

Basic data for each player

#### Sample

```json
[
  {
    "steam_id": "76561198024494988",
    "name": "Icewind  demostf",
    "avatar": "https://avatars.akamai.steamstatic.com/75b84075b70535c5cfb3499af03b3e4e7a7b556f_full.jpg",
    "country": "nl"
  }
]
```

## membership.json

Historical team membership data

#### Sample

```json
[
  {
    "team_id": 7861,
    "steam_id": "76561198024494988",
    "role": "leader",
    "joined": "2013/08/09",
    "left": null
  }
]
```

## matches.json

Match data.

Due to limitations of the data available on ugc, the asia region is currently not included.

#### Sample

```json
[
  {
    "id": 62613,
    "team_home": 7861,
    "team_away": 9387,
    "score_home": 2,
    "score_away": 4,
    "map": "koth_lakeside_final",
    "season": 16,
    "week": 8,
    "default_date": "2015/07/20",
    "format": "9v9"
  }
]
```
