Just another ifconfig clone made in actix-web, this is Rust!

Since I just started learning rust, this code is kinda horrible right know, I'll try to improve it as I learn more about the language. For now, I should focus on optimizing string handling and error handling.

Regardless, it's already `4 times faster` than ifconfig.me! Check [benchmarks](#benchmarks) for more info.

# Usage

Since I don't have a specific domain for this project, I'm using the one I already have to host this service.

Open [ip.boringcalculator.com](https://ip.boringcalculator.com) in your browser or use `curl` to get your IP address.

```bash
curl ip.boringcalculator.com -4
```

> 34.16.196.246

```bash
curl ip.boringcalculator.com -6
```

> 2600:1900:4180:10c::

```bash
curl ip.boringcalculator.com/all
```

```
ip_address: 2600:1900:4180:10c::
accept: */*
user-agent: curl/7.88.1
host: ip.boringcalculator.com
method: GET
```

```bash
curl ip.boringcalculator.com/all.json
```

```json
{
  "accept": "*/*",
  "host": "ip.boringcalculator.com",
  "ip_address": "2600:1900:4180:10c::",
  "method": "GET",
  "user-agent": "curl/7.88.1"
}
```

# Improvements to do

- Add more information about the user's IP address, like the country, city, and ISP.

# Benchmarks

I am using [hey](https://github.com/rakyll/hey) to benchmark the service.

## ifconfig-rs

```bash
hey -z 2s -H  "User-Agent: curl" https://ip.boringcalculator.com
```

```
Summary:
  Total:        2.0160 secs
  Slowest:      0.0748 secs
  Fastest:      0.0138 secs
  Average:      0.0182 secs
  Requests/sec: 2741.5781


Response time histogram:
  0.014 [1]     |
  0.020 [4872]  |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.026 [506]   |■■■■
  0.032 [57]    |
  0.038 [34]    |
  0.044 [7]     |
  0.050 [0]     |
  0.057 [0]     |
  0.063 [10]    |
  0.069 [19]    |
  0.075 [21]    |


Latency distribution:
  10% in 0.0152 secs
  25% in 0.0161 secs
  50% in 0.0173 secs
  75% in 0.0186 secs
  90% in 0.0201 secs
  95% in 0.0221 secs
  99% in 0.0386 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0004 secs, 0.0138 secs, 0.0748 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0141 secs
  req write:    0.0000 secs, 0.0000 secs, 0.0003 secs
  resp wait:    0.0177 secs, 0.0137 secs, 0.0422 secs
  resp read:    0.0001 secs, 0.0000 secs, 0.0008 secs

Status code distribution:
  [200] 5527 responses

```

## ifconfig.me

```bash
hey -z 2s -H  "User-Agent: curl" https://ifconfig.me
```

```
Summary:
  Total:        2.0610 secs
  Slowest:      0.1718 secs
  Fastest:      0.0585 secs
  Average:      0.0697 secs
  Requests/sec: 707.4178

  Total data:   15549570 bytes
  Size/request: 10665 bytes

Response time histogram:
  0.059 [1]     |
  0.070 [936]   |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.081 [441]   |■■■■■■■■■■■■■■■■■■■
  0.092 [14]    |■
  0.104 [26]    |■
  0.115 [4]     |
  0.126 [30]    |■
  0.138 [0]     |
  0.149 [0]     |
  0.160 [5]     |
  0.172 [1]     |


Latency distribution:
  10% in 0.0613 secs
  25% in 0.0631 secs
  50% in 0.0663 secs
  75% in 0.0732 secs
  90% in 0.0767 secs
  95% in 0.0864 secs
  99% in 0.1209 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0013 secs, 0.0585 secs, 0.1718 secs
  DNS-lookup:   0.0005 secs, 0.0000 secs, 0.0190 secs
  req write:    0.0000 secs, 0.0000 secs, 0.0008 secs
  resp wait:    0.0643 secs, 0.0560 secs, 0.1093 secs
  resp read:    0.0036 secs, 0.0000 secs, 0.0441 secs

Status code distribution:
  [200] 1458 responses
```
