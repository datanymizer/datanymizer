+++
title = "Datanymizer"

# The homepage contents
[extra]
lead = '<b>Datanymizer</b> is a powerful database anonymizer with flexible rules. Written in Rust.'
url = "/docs/getting-started/introduction//"
url_button = "Get started"
repo_version = "GitHub v0.2.0"
repo_license = "Open-source MIT License."
repo_url = "https://github.com/datanymizer/datanymizer"


[[extra.list]]
title = "Available rules"
content = 'More than 70 rules in total, like: <code>email</code>, <code>first_name</code>, <code>city</code>, <code>phone</code>, <code>password</code>'

[[extra.list]]
title = "Database-native dumps"
content = 'You can import or process your dump with supported database without 3rd-party importers.'

[[extra.list]]
title = "Tables filter"
content = 'You can specify which tables you choose or ignore for making dump.'

[[extra.list]]
title = "Dump conditions and limit"
content = 'You can specify conditions (SQL <code>WHERE</code> statement) and limit for dumped data per table'

[[extra.list]]
title = "Transform conditions and limit"
content = 'As the additional option, you can specify SQL-conditions that define which rows will be transformed (anonymized)'

[[extra.list]]
title = "Global variables"
content = 'You can specify global variables available from any <code>template</code> rule.'

[[extra.list]]
title = "Uniqueness"
content = 'You can specify that result values must be unique (they are not unique by default). Uniqueness is ensured by re-generating values when they are same.'

[[extra.list]]
title = "Locales"
content = 'You can specify the locale for individual rules. The default locale is <code>EN</code> but you can specify a different default locale.'

[[extra.list]]
title = "Referencing row values from templates"
content = 'You can reference values of other row fields in templates.'

+++
