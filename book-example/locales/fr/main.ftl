book-title = documentation mdBook
localized-chapter-title = Exemple localisé
simple = texte simple
reference = texte simple avec une référence: { -something }

parameter = texte avec une { $param }
parameter2 = texte une { $param1 } seconde { $param2 }

crabs =
    { $crabs ->
        [one] Il y a un crabe.
       *[other] Vous avez { $crabs } crabes.
    }

# no fallback
