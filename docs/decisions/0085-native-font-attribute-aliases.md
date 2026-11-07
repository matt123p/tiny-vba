# ADR 0085: Published Font attribute aliases

Date: 2026-11-07. Status: shared acceptance passed; affected host acceptance in progress. M7A.4 remains open.

The Forms interface exposes FontName, FontSize, FontWeight, FontBold, FontItalic,
FontUnderline and FontStrikethru on ten text control families: Label, TextBox,
CommandButton, CheckBox, OptionButton, ToggleButton, ListBox, ComboBox, TabStrip
and MultiPage. These ordinary properties delegate to the canonical Font object.
They create no duplicate state, VM behavior, host operation or resource budget.
IOptionFrame (UserForm/Frame), IPage and non-text controls do not gain aliases.
RefEdit alias coverage is not admitted from a different control's interface.
Tab/Page shortcut generation reserves actual owner Font alias names.

FontSize is Currency, FontWeight Integer, FontName String, and the four style
flags Boolean. Return/argument coercion and Font attribute validation remain in
the existing shared path. Aliases follow explicitly replaced Font references;
aliases of old Font objects retain their prior values. Inspection/rendering still
reads fields without executing these properties. Stop/Continue and source/resource
Reset follow ordinary frames. No font acquisition or global style mutation occurs.

Two additional authored model cases check exact types/values, coupled weight/bold,
replacement, old aliases, Stop/Continue, invalid values, overflow and unrelated
owner member absence. A GPU equality case compares these aliases with an assigned
Font. The broader member/appearance/platform audit remains open; fresh Office
comparisons remain M8.5 NOT_RUN.

References:

- [TextBox properties](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.textboxclass?view=office-pia)
- [FontSize](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.textboxclass.fontsize?view=office-pia)
- [FontWeight](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.textboxclass.fontweight?view=office-pia)
- [MultiPage properties](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.multipageclass?view=office-pia)
- [Frame properties](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.frameclass?view=office-pia)
- [Page properties](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.page?view=office-pia)
