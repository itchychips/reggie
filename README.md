# Reggie

## Synopsis

`reggie` is a fast registry searcher for Windows.

Currently, it only searches for keys by name.  It parallelizes via
[rayon](https://docs.rs/rayon/latest/rayon/), though it has the option of
running in single-threaded mode without it.

Run `reggie -h` for help.

## Examples

List registry hives available:

```
$ reggie -l
HKLM, HKEY_LOCAL_MACHINE, -2147483646
HKCR, HKEY_CLASSES_ROOT, -2147483648
HKCC, HKEY_CURRENT_CONFIG, -2147483643
HKCU, HKEY_CURRENT_USER, -2147483647
HKCULL, HKEY_CURRENT_USER_LOCAL_SETTINGS, -2147483641
HKDD, HKEY_DYN_DATA, -2147483642
HKPD, HKEY_PERFORMANCE_DATA, -2147483644
HKPL, HKEY_PERFORMANCE_NLSTEXT, -2147483552
HKPT, HKEY_PERFORMANCE_TEXT, -2147483568
HKU, HKEY_USERS, -2147483645
```

List registry keys in the `HKEY_LOCAL_MACHINE` hive by default:

```
$ reggie -p | head
HKLM
HKLM\HARDWARE
HKLM\HARDWARE\ACPI
HKLM\HARDWARE\ACPI\DSDT
HKLM\HARDWARE\ACPI\DSDT\ALASKA
HKLM\HARDWARE\ACPI\DSDT\ALASKA\A_M_I_
HKLM\HARDWARE\ACPI\DSDT\ALASKA\A_M_I_\01072009
HKLM\HARDWARE\ACPI\FACS
HKLM\HARDWARE\ACPI\FADT
HKLM\HARDWARE\ACPI\FADT\ALASKA
Error: Os { code: 232, kind: BrokenPipe, message: "The pipe is being closed." }

$ reggie -H HKLM -p | head
HKLM
HKLM\HARDWARE
HKLM\HARDWARE\ACPI
HKLM\HARDWARE\ACPI\DSDT
HKLM\HARDWARE\ACPI\DSDT\ALASKA
HKLM\HARDWARE\ACPI\DSDT\ALASKA\A_M_I_
HKLM\HARDWARE\ACPI\DSDT\ALASKA\A_M_I_\01072009
HKLM\HARDWARE\ACPI\FACS
HKLM\HARDWARE\ACPI\FADT
HKLM\HARDWARE\ACPI\FADT\ALASKA
Error: Os { code: 232, kind: BrokenPipe, message: "The pipe is being closed." }
```

(note: Examples are running in a MINGW64 environment)

Search for registry keys that case insensitively contain "mozilla" within the
`HKEY_LOCAL_MACHINE` hive:

```
$ reggie -H HKLM -f "mozilla"
HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Mozilla Firefox 112.0.1 (x64 en-US)
HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\MozillaMaintenanceService
HKLM\SOFTWARE\Mozilla
HKLM\SOFTWARE\Mozilla\Firefox
HKLM\SOFTWARE\Mozilla\Firefox\TaskBarIDs
HKLM\SOFTWARE\Mozilla\MaintenanceService
HKLM\SOFTWARE\Mozilla\MaintenanceService\f9b87e891978e3145f0f8f9953eadc00
HKLM\SOFTWARE\Mozilla\MaintenanceService\f9b87e891978e3145f0f8f9953eadc00\0
HKLM\SOFTWARE\Mozilla\MaintenanceService\f9b87e891978e3145f0f8f9953eadc00\1
HKLM\SOFTWARE\Mozilla\Mozilla Firefox
HKLM\SOFTWARE\Mozilla\Mozilla Firefox 112.0.1
HKLM\SOFTWARE\Mozilla\Mozilla Firefox 112.0.1\bin
HKLM\SOFTWARE\Mozilla\Mozilla Firefox 112.0.1\extensions
HKLM\SOFTWARE\Mozilla\Mozilla Firefox\112.0.1 (x64 en-US)
HKLM\SOFTWARE\Mozilla\Mozilla Firefox\112.0.1 (x64 en-US)\Main
HKLM\SOFTWARE\Mozilla\Mozilla Firefox\112.0.1 (x64 en-US)\Uninstall
HKLM\SOFTWARE\Mozilla\NativeMessagingHosts
HKLM\SOFTWARE\Mozilla\NativeMessagingHosts\com.microsoft.defender.browser_extension.native_message_host
HKLM\SOFTWARE\mozilla.org
HKLM\SOFTWARE\mozilla.org\Mozilla
HKLM\SYSTEM\ControlSet001\Services\MozillaMaintenance
HKLM\SYSTEM\CurrentControlSet\Services\MozillaMaintenance
```

Same, but case sensitively:

```
$ reggie -H HKLM -f "(?-i)mozilla"
HKLM\SOFTWARE\mozilla.org
HKLM\SOFTWARE\mozilla.org\Mozilla
```

The regular expression filter, if not given, defaults to "" (which will match
everything), and if given, is prepended with "(?i)" to turn off case
sensitivity.

See [regex crate
documentation](https://docs.rs/regex/latest/regex/#grouping-and-flags) for more
information on how to customize regular expressions.

Benchmark registry searches by specifying the count and time switches:

```
$ reggie -H HKLM -ct
There are 400790 keys in HKLM.
Took 2.4652453 seconds
162576 keys/second
```

Change the number of threads used to search:

```
$ reggie -H HKCR -ct -T 16
There are 122138 keys in HKCR.
Took 6.5700107 seconds
18590 keys/second

$ reggie -H HKCR -ct -T 2
There are 176456 keys in HKCR.
Took 4.010996 seconds
43993 keys/second

$ reggie -H HKCR -ct -T 1
There are 176485 keys in HKCR.
Took 5.0277161 seconds
35102 keys/second
```

`HKEY_CLASSES_ROOT` seems to be slower than HKLM, despite the fewer number of
keys.  Additionally, the key counts seem to change a lot.  The author blames
the lack of their knowledge on what `HKEY_CLASSES_ROOT` actually holds.

Don't use the threaded backend, which generally has the same performance as
setting the number of threads to 1:

```
$ reggie -H HKLM -ct -B v1
There are 400790 keys in HKLM.
Took 5.7450432 seconds
69762 keys/second

$ reggie -H HKLM -ct -T 1
There are 400790 keys in HKLM.
Took 5.9761852 seconds
67064 keys/second
```

## License

See LICENSE.txt, or navigate to <https://www.gnu.org/licenses/gpl-3.0.en.html>.
