# Linklist

_Website internal link lister_

This tool lists all local page links from within a webpage by looking for all href tags within the HTML source.

Linklister is built in Rust and provides a quick and easy to use tool for getting all links which point to other pages on
the given domain. The output can be customised to include links to files.

## Usage

### Arguments
|**Argument**|**Description**|
|------------|---------------|
|`--color`   |Colorise outputs by category (page/file/other)|
|`-f,--file-type=<suffix>`|Include files (by default only pages), filter by file suffix else use 'all' to list all files.|
|`-p,--path`|List links as relative paths.|
|`-h,--help`|Display command help.|
|`-V,--version`|Version information.|

```sh
$ linklist google.com
http://google.com/intl/en/ads/
http://google.com/intl/en/policies/privacy/
http://google.com/intl/en/policies/terms/
http://google.com/preferences
http://google.com/services/
```