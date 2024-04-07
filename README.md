# ilj-to-gc

[Fujitsu の iLiswave-J](https://www.fujitsu.com/jp/solutions/industry/education/campus/library/) が導入された WebOPAC で、貸し出した本の返却期限日を自動で Google Calendar に登録するツールです。動作は保証しません。

## Usage

`config.toml` に以下のように設定します。

| key | value |
| --- | ----- |
| `base_url` | WebOPAC のベース URL |
| `userid` | WebOPAC のユーザー ID |
| `password` | WebOPAC のパスワード |
| `google_private_key_id` | Google service account の `private_key_id` |
| `google_private_key_id` | Google service account の `private_key` |
| `google_client_email` | Google service account の `private_client_email` |
| `calendar_id` | Google Calendar のカレンダー ID |
