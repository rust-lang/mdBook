-- ANCHOR: eqNonEmpty  
instance Eq (NonEmpty a) where
  eq (NonEmpty x xs) (NonEmpty y ys) = x == y && xs == ys    
-- ANCHOR_END: eqNonEmpty