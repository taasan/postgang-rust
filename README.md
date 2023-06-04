# Norwegian mailbox delivery dates calendar

A command line interface for generating an iCal version of [Bring's API for mailbox delivery dates](https://developer.bring.com/api/postal-code/#get-mailbox-delivery-dates-at-postal-code-get)
or from a JSON file.

## Help

```
Usage: postgang [OPTIONS] --code <CODE> <COMMAND>

Commands:
  api   Get delivery dates from Bring API
  file  Get delivery dates from JSON file
  help  Print this message or the help of the given subcommand(s)

Options:
      --code <CODE>      Postal code
      --output <OUTPUT>  File path
  -h, --help             Print help
  -V, --version          Print version
```

## Api

```
Get delivery dates from Bring API

Usage: postgang --code <CODE> api --api-uid <API_UID> --api-key <API_KEY>

Options:
      --api-uid <API_UID>  [env: POSTGANG_API_UID]
      --api-key <API_KEY>  [env: POSTGANG_API_KEY]
  -h, --help               Print help
```

## File

```
Get delivery dates from JSON file

Usage: postgang --code <CODE> file [INPUT]

Arguments:
  [INPUT]  File path, read from stdin of omitted

Options:
  -h, --help  Print help
```

## Example output

```ical
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Aasan//Aasan Postgang//EN
CALSCALE:GREGORIAN
METHOD:PUBLISH
BEGIN:VEVENT
DTEND;VALUE=DATE:20230207
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20230206
SUMMARY:7530: Posten kommer mandag 6.
TRANSP:TRANSPARENT
UID:postgang-7530-2023-02-06
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20230209
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20230208
SUMMARY:7530: Posten kommer onsdag 8.
TRANSP:TRANSPARENT
UID:postgang-7530-2023-02-08
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20230211
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20230210
SUMMARY:7530: Posten kommer fredag 10.
TRANSP:TRANSPARENT
UID:postgang-7530-2023-02-10
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20230215
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20230214
SUMMARY:7530: Posten kommer tirsdag 14.
TRANSP:TRANSPARENT
UID:postgang-7530-2023-02-14
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20230217
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20230216
SUMMARY:7530: Posten kommer torsdag 16.
TRANSP:TRANSPARENT
UID:postgang-7530-2023-02-16
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20230221
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20230220
SUMMARY:7530: Posten kommer mandag 20.
TRANSP:TRANSPARENT
UID:postgang-7530-2023-02-20
URL:https://www.posten.no/levering-av-post/
END:VEVENT
END:VCALENDAR
```
