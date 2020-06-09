book-title = mdBook documentation
localized-chapter-title = Localized Example
simple = simple text
reference = simple text with a reference: { -something }

parameter = text with a { $param }
parameter2 = text one { $param1 } second { $param2 }

email = text with an EMAIL("example@example.org")

crabs =
    { $crabs ->
        [one] There's one crab.
        *[other] You have { $crabs } crabs.
    }

fallback = this should fall back
