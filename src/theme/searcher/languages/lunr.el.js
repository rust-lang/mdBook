/*!
 * Lunr languages, `Greek` language
 * https://github.com/MihaiValentin/lunr-languages
 *
 * Copyright 2023, Panos Bariamis
 * http://www.mozilla.org/MPL/
 */
/*!
 * based on
 * Snowball JavaScript Library v0.3
 * http://code.google.com/p/urim/
 * http://snowball.tartarus.org/
 *
 * Copyright 2010, Oleg Mazko
 * http://www.mozilla.org/MPL/
 */

/**
 * export the module via AMD, CommonJS or as a browser global
 * Export code from https://github.com/umdjs/umd/blob/master/returnExports.js
 */
;
(function(root, factory) {
  if (typeof define === 'function' && define.amd) {
    // AMD. Register as an anonymous module.
    define(factory)
  } else if (typeof exports === 'object') {
    /**
     * Node. Does not work with strict CommonJS, but
     * only CommonJS-like environments that support module.exports,
     * like Node.
     */
    module.exports = factory()
  } else {
    // Browser globals (root is window)
    factory()(root.lunr);
  }
}(this, function() {
  /**
   * Just return a value to define the module export.
   * This example returns an object, but the module
   * can return a function as the exported value.
   */
  return function(lunr) {
    /* throw error if lunr is not yet included */
    if ('undefined' === typeof lunr) {
      throw new Error('Lunr is not present. Please include / require Lunr before this script.');
    }

    /* throw error if lunr stemmer support is not yet included */
    if ('undefined' === typeof lunr.stemmerSupport) {
      throw new Error('Lunr stemmer support is not present. Please include / require Lunr stemmer support before this script.');
    }

    /* register specific locale function */
    lunr.el = function() {
      this.pipeline.reset();

      if (this.searchPipeline === undefined) {
        this.pipeline.add(
          lunr.el.trimmer,
          lunr.el.normilizer
        );
      }

      this.pipeline.add(
        lunr.el.stopWordFilter,
        lunr.el.stemmer
      );

      // for lunr version 2
      // this is necessary so that every searched word is also stemmed before
      // in lunr <= 1 this is not needed, as it is done using the normal pipeline
      if (this.searchPipeline) {
        this.searchPipeline.reset();
        this.searchPipeline.add(
          lunr.el.stemmer
        );
      }
    };

    /* lunr trimmer function */
    lunr.el.wordCharacters = "A-Za-zΑαΒβΓγΔδΕεΖζΗηΘθΙιΚκΛλΜμΝνΞξΟοΠπΡρΣσςΤτΥυΦφΧχΨψΩωΆάΈέΉήΊίΌόΎύΏώΪΐΫΰΐΰ";
    lunr.el.trimmer = lunr.trimmerSupport.generateTrimmer(lunr.el.wordCharacters);

    lunr.Pipeline.registerFunction(lunr.el.trimmer, 'trimmer-el');

    /* lunr stemmer function */
    lunr.el.stemmer = (function() {
      var stepOneExceptions = {
        'ΦΑΓΙΑ': 'ΦΑ',
        'ΦΑΓΙΟΥ': 'ΦΑ',
        'ΦΑΓΙΩΝ': 'ΦΑ',
        'ΣΚΑΓΙΑ': 'ΣΚΑ',
        'ΣΚΑΓΙΟΥ': 'ΣΚΑ',
        'ΣΚΑΓΙΩΝ': 'ΣΚΑ',
        'ΣΟΓΙΟΥ': 'ΣΟ',
        'ΣΟΓΙΑ': 'ΣΟ',
        'ΣΟΓΙΩΝ': 'ΣΟ',
        'ΤΑΤΟΓΙΑ': 'ΤΑΤΟ',
        'ΤΑΤΟΓΙΟΥ': 'ΤΑΤΟ',
        'ΤΑΤΟΓΙΩΝ': 'ΤΑΤΟ',
        'ΚΡΕΑΣ': 'ΚΡΕ',
        'ΚΡΕΑΤΟΣ': 'ΚΡΕ',
        'ΚΡΕΑΤΑ': 'ΚΡΕ',
        'ΚΡΕΑΤΩΝ': 'ΚΡΕ',
        'ΠΕΡΑΣ': 'ΠΕΡ',
        'ΠΕΡΑΤΟΣ': 'ΠΕΡ',
        'ΠΕΡΑΤΑ': 'ΠΕΡ',
        'ΠΕΡΑΤΩΝ': 'ΠΕΡ',
        'ΤΕΡΑΣ': 'ΤΕΡ',
        'ΤΕΡΑΤΟΣ': 'ΤΕΡ',
        'ΤΕΡΑΤΑ': 'ΤΕΡ',
        'ΤΕΡΑΤΩΝ': 'ΤΕΡ',
        'ΦΩΣ': 'ΦΩ',
        'ΦΩΤΟΣ': 'ΦΩ',
        'ΦΩΤΑ': 'ΦΩ',
        'ΦΩΤΩΝ': 'ΦΩ',
        'ΚΑΘΕΣΤΩΣ': 'ΚΑΘΕΣΤ',
        'ΚΑΘΕΣΤΩΤΟΣ': 'ΚΑΘΕΣΤ',
        'ΚΑΘΕΣΤΩΤΑ': 'ΚΑΘΕΣΤ',
        'ΚΑΘΕΣΤΩΤΩΝ': 'ΚΑΘΕΣΤ',
        'ΓΕΓΟΝΟΣ': 'ΓΕΓΟΝ',
        'ΓΕΓΟΝΟΤΟΣ': 'ΓΕΓΟΝ',
        'ΓΕΓΟΝΟΤΑ': 'ΓΕΓΟΝ',
        'ΓΕΓΟΝΟΤΩΝ': 'ΓΕΓΟΝ',
        'ΕΥΑ': 'ΕΥ'
      };
      var protectedWords = [
        'ΑΚΡΙΒΩΣ',
        'ΑΛΑ',
        'ΑΛΛΑ',
        'ΑΛΛΙΩΣ',
        'ΑΛΛΟΤΕ',
        'ΑΜΑ',
        'ΑΝΩ',
        'ΑΝΑ',
        'ΑΝΑΜΕΣΑ',
        'ΑΝΑΜΕΤΑΞΥ',
        'ΑΝΕΥ',
        'ΑΝΤΙ',
        'ΑΝΤΙΠΕΡΑ',
        'ΑΝΤΙΟ',
        'ΑΞΑΦΝΑ',
        'ΑΠΟ',
        'ΑΠΟΨΕ',
        'ΑΡΑ',
        'ΑΡΑΓΕ',
        'ΑΥΡΙΟ',
        'ΑΦΟΙ',
        'ΑΦΟΥ',
        'ΑΦΟΤΟΥ',
        'ΒΡΕ',
        'ΓΕΙΑ',
        'ΓΙΑ',
        'ΓΙΑΤΙ',
        'ΓΡΑΜΜΑ',
        'ΔΕΗ',
        'ΔΕΝ',
        'ΔΗΛΑΔΗ',
        'ΔΙΧΩΣ',
        'ΔΥΟ',
        'ΕΑΝ',
        'ΕΓΩ',
        'ΕΔΩ',
        'ΕΔΑ',
        'ΕΙΘΕ',
        'ΕΙΜΑΙ',
        'ΕΙΜΑΣΤΕ',
        'ΕΙΣΑΙ',
        'ΕΙΣΑΣΤΕ',
        'ΕΙΝΑΙ',
        'ΕΙΣΤΕ',
        'ΕΙΤΕ',
        'ΕΚΕΙ',
        'ΕΚΟ',
        'ΕΛΑ',
        'ΕΜΑΣ',
        'ΕΜΕΙΣ',
        'ΕΝΤΕΛΩΣ',
        'ΕΝΤΟΣ',
        'ΕΝΤΩΜΕΤΑΞΥ',
        'ΕΝΩ',
        'ΕΞΙ',
        'ΕΞΙΣΟΥ',
        'ΕΞΗΣ',
        'ΕΞΩ',
        'ΕΟΚ',
        'ΕΠΑΝΩ',
        'ΕΠΕΙΔΗ',
        'ΕΠΕΙΤΑ',
        'ΕΠΙ',
        'ΕΠΙΣΗΣ',
        'ΕΠΟΜΕΝΩΣ',
        'ΕΠΤΑ',
        'ΕΣΑΣ',
        'ΕΣΕΙΣ',
        'ΕΣΤΩ',
        'ΕΣΥ',
        'ΕΣΩ',
        'ΕΤΣΙ',
        'ΕΥΓΕ',
        'ΕΦΕ',
        'ΕΦΕΞΗΣ',
        'ΕΧΤΕΣ',
        'ΕΩΣ',
        'ΗΔΗ',
        'ΗΜΙ',
        'ΗΠΑ',
        'ΗΤΟΙ',
        'ΘΕΣ',
        'ΙΔΙΩΣ',
        'ΙΔΗ',
        'ΙΚΑ',
        'ΙΣΩΣ',
        'ΚΑΘΕ',
        'ΚΑΘΕΤΙ',
        'ΚΑΘΟΛΟΥ',
        'ΚΑΘΩΣ',
        'ΚΑΙ',
        'ΚΑΝ',
        'ΚΑΠΟΤΕ',
        'ΚΑΠΟΥ',
        'ΚΑΤΑ',
        'ΚΑΤΙ',
        'ΚΑΤΟΠΙΝ',
        'ΚΑΤΩ',
        'ΚΕΙ',
        'ΚΙΧ',
        'ΚΚΕ',
        'ΚΟΛΑΝ',
        'ΚΥΡΙΩΣ',
        'ΚΩΣ',
        'ΜΑΚΑΡΙ',
        'ΜΑΛΙΣΤΑ',
        'ΜΑΛΛΟΝ',
        'ΜΑΙ',
        'ΜΑΟ',
        'ΜΑΟΥΣ',
        'ΜΑΣ',
        'ΜΕΘΑΥΡΙΟ',
        'ΜΕΣ',
        'ΜΕΣΑ',
        'ΜΕΤΑ',
        'ΜΕΤΑΞΥ',
        'ΜΕΧΡΙ',
        'ΜΗΔΕ',
        'ΜΗΝ',
        'ΜΗΠΩΣ',
        'ΜΗΤΕ',
        'ΜΙΑ',
        'ΜΙΑΣ',
        'ΜΙΣ',
        'ΜΜΕ',
        'ΜΟΛΟΝΟΤΙ',
        'ΜΟΥ',
        'ΜΠΑ',
        'ΜΠΑΣ',
        'ΜΠΟΥΦΑΝ',
        'ΜΠΡΟΣ',
        'ΝΑΙ',
        'ΝΕΣ',
        'ΝΤΑ',
        'ΝΤΕ',
        'ΞΑΝΑ',
        'ΟΗΕ',
        'ΟΚΤΩ',
        'ΟΜΩΣ',
        'ΟΝΕ',
        'ΟΠΑ',
        'ΟΠΟΥ',
        'ΟΠΩΣ',
        'ΟΣΟ',
        'ΟΤΑΝ',
        'ΟΤΕ',
        'ΟΤΙ',
        'ΟΥΤΕ',
        'ΟΧΙ',
        'ΠΑΛΙ',
        'ΠΑΝ',
        'ΠΑΝΟ',
        'ΠΑΝΤΟΤΕ',
        'ΠΑΝΤΟΥ',
        'ΠΑΝΤΩΣ',
        'ΠΑΝΩ',
        'ΠΑΡΑ',
        'ΠΕΡΑ',
        'ΠΕΡΙ',
        'ΠΕΡΙΠΟΥ',
        'ΠΙΑ',
        'ΠΙΟ',
        'ΠΙΣΩ',
        'ΠΛΑΙ',
        'ΠΛΕΟΝ',
        'ΠΛΗΝ',
        'ΠΟΤΕ',
        'ΠΟΥ',
        'ΠΡΟ',
        'ΠΡΟΣ',
        'ΠΡΟΧΤΕΣ',
        'ΠΡΟΧΘΕΣ',
        'ΡΟΔΙ',
        'ΠΩΣ',
        'ΣΑΙ',
        'ΣΑΣ',
        'ΣΑΝ',
        'ΣΕΙΣ',
        'ΣΙΑ',
        'ΣΚΙ',
        'ΣΟΙ',
        'ΣΟΥ',
        'ΣΡΙ',
        'ΣΥΝ',
        'ΣΥΝΑΜΑ',
        'ΣΧΕΔΟΝ',
        'ΤΑΔΕ',
        'ΤΑΞΙ',
        'ΤΑΧΑ',
        'ΤΕΙ',
        'ΤΗΝ',
        'ΤΗΣ',
        'ΤΙΠΟΤΑ',
        'ΤΙΠΟΤΕ',
        'ΤΙΣ',
        'ΤΟΝ',
        'ΤΟΤΕ',
        'ΤΟΥ',
        'ΤΟΥΣ',
        'ΤΣΑ',
        'ΤΣΕ',
        'ΤΣΙ',
        'ΤΣΟΥ',
        'ΤΩΝ',
        'ΥΠΟ',
        'ΥΠΟΨΗ',
        'ΥΠΟΨΙΝ',
        'ΥΣΤΕΡΑ',
        'ΦΕΤΟΣ',
        'ΦΙΣ',
        'ΦΠΑ',
        'ΧΑΦ',
        'ΧΘΕΣ',
        'ΧΤΕΣ',
        'ΧΩΡΙΣ',
        'ΩΣ',
        'ΩΣΑΝ',
        'ΩΣΟΤΟΥ',
        'ΩΣΠΟΥ',
        'ΩΣΤΕ',
        'ΩΣΤΟΣΟ'
      ];

      var alphabet = new RegExp('^[ΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩ]+$');

      function isGreek(word) {
        return alphabet.test(word);
      }

      function endsInVowel(word) {
        return /[ΑΕΗΙΟΥΩ]$/.test(word);
      }

      function endsInVowel2(word) {
        return /[ΑΕΗΙΟΩ]$/.test(word);
      }

      function stem(word) {

        var stemmedWord = word;

        if (word.length < 3) {
          return stemmedWord;
        }

        if (!isGreek(word)) {
          return stemmedWord;
        }

        if (protectedWords.indexOf(word) >= 0) {
          return stemmedWord;
        }

        //step 1
        var stepOneRegExp = new RegExp('(.*)(' + Object.keys(stepOneExceptions).join('|') + ')$');
        var match = stepOneRegExp.exec(stemmedWord);

        if (match !== null) {
          stemmedWord = match[1] + stepOneExceptions[match[2]];
        }
        //step 2
        //2a
        if ((match = /^(.+?)(ΑΔΕΣ|ΑΔΩΝ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (!/(ΟΚ|ΜΑΜ|ΜΑΝ|ΜΠΑΜΠ|ΠΑΤΕΡ|ΓΙΑΓΙ|ΝΤΑΝΤ|ΚΥΡ|ΘΕΙ|ΠΕΘΕΡ|ΜΟΥΣΑΜ|ΚΑΠΛΑΜ|ΠΑΡ|ΨΑΡ|ΤΖΟΥΡ|ΤΑΜΠΟΥΡ|ΓΑΛΑΤ|ΦΑΦΛΑΤ)$/.test(match[1])) {
            stemmedWord += 'ΑΔ';
          }
        }

        //2b
        if ((match = /^(.+?)(ΕΔΕΣ|ΕΔΩΝ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/(ΟΠ|ΙΠ|ΕΜΠ|ΥΠ|ΓΗΠ|ΔΑΠ|ΚΡΑΣΠ|ΜΙΛ)$/.test(match[1])) {
            stemmedWord += 'ΕΔ';
          }
        }

        //2c
        if ((match = /^(.+?)(ΟΥΔΕΣ|ΟΥΔΩΝ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/(ΑΡΚ|ΚΑΛΙΑΚ|ΠΕΤΑΛ|ΛΙΧ|ΠΛΕΞ|ΣΚ|Σ|ΦΛ|ΦΡ|ΒΕΛ|ΛΟΥΛ|ΧΝ|ΣΠ|ΤΡΑΓ|ΦΕ)$/.test(match[1])) {
            stemmedWord += 'ΟΥΔ';
          }
        }

        //2d
        if ((match = /^(.+?)(ΕΩΣ|ΕΩΝ|ΕΑΣ|ΕΑ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(Θ|Δ|ΕΛ|ΓΑΛ|Ν|Π|ΙΔ|ΠΑΡ|ΣΤΕΡ|ΟΡΦ|ΑΝΔΡ|ΑΝΤΡ)$/.test(match[1])) {
            stemmedWord += 'Ε';
          }
        }

        //step 3
        //3a
        if ((match = /^(.+?)(ΕΙΟ|ΕΙΟΣ|ΕΙΟΙ|ΕΙΑ|ΕΙΑΣ|ΕΙΕΣ|ΕΙΟΥ|ΕΙΟΥΣ|ΕΙΩΝ)$/.exec(stemmedWord)) !== null && match[1].length > 4) {
          stemmedWord = match[1];
        }

        //3b
        if ((match = /^(.+?)(ΙΟΥΣ|ΙΑΣ|ΙΕΣ|ΙΟΣ|ΙΟΥ|ΙΟΙ|ΙΩΝ|ΙΟΝ|ΙΑ|ΙΟ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (endsInVowel(stemmedWord) || stemmedWord.length < 2 || /^(ΑΓ|ΑΓΓΕΛ|ΑΓΡ|ΑΕΡ|ΑΘΛ|ΑΚΟΥΣ|ΑΞ|ΑΣ|Β|ΒΙΒΛ|ΒΥΤ|Γ|ΓΙΑΓ|ΓΩΝ|Δ|ΔΑΝ|ΔΗΛ|ΔΗΜ|ΔΟΚΙΜ|ΕΛ|ΖΑΧΑΡ|ΗΛ|ΗΠ|ΙΔ|ΙΣΚ|ΙΣΤ|ΙΟΝ|ΙΩΝ|ΚΙΜΩΛ|ΚΟΛΟΝ|ΚΟΡ|ΚΤΗΡ|ΚΥΡ|ΛΑΓ|ΛΟΓ|ΜΑΓ|ΜΠΑΝ|ΜΠΡ|ΝΑΥΤ|ΝΟΤ|ΟΠΑΛ|ΟΞ|ΟΡ|ΟΣ|ΠΑΝΑΓ|ΠΑΤΡ|ΠΗΛ|ΠΗΝ|ΠΛΑΙΣ|ΠΟΝΤ|ΡΑΔ|ΡΟΔ|ΣΚ|ΣΚΟΡΠ|ΣΟΥΝ|ΣΠΑΝ|ΣΤΑΔ|ΣΥΡ|ΤΗΛ|ΤΙΜ|ΤΟΚ|ΤΟΠ|ΤΡΟΧ|ΦΙΛ|ΦΩΤ|Χ|ΧΙΛ|ΧΡΩΜ|ΧΩΡ)$/.test(match[1])) {
            stemmedWord += 'Ι';
          }
          if (/^(ΠΑΛ)$/.test(match[1])) {
            stemmedWord += 'ΑΙ';
          }
        }

        //step 4
        if ((match = /^(.+?)(ΙΚΟΣ|ΙΚΟΝ|ΙΚΕΙΣ|ΙΚΟΙ|ΙΚΕΣ|ΙΚΟΥΣ|ΙΚΗ|ΙΚΗΣ|ΙΚΟ|ΙΚΑ|ΙΚΟΥ|ΙΚΩΝ|ΙΚΩΣ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (endsInVowel(stemmedWord) || /^(ΑΔ|ΑΛ|ΑΜΑΝ|ΑΜΕΡ|ΑΜΜΟΧΑΛ|ΑΝΗΘ|ΑΝΤΙΔ|ΑΠΛ|ΑΤΤ|ΑΦΡ|ΒΑΣ|ΒΡΩΜ|ΓΕΝ|ΓΕΡ|Δ|ΔΙΚΑΝ|ΔΥΤ|ΕΙΔ|ΕΝΔ|ΕΞΩΔ|ΗΘ|ΘΕΤ|ΚΑΛΛΙΝ|ΚΑΛΠ|ΚΑΤΑΔ|ΚΟΥΖΙΝ|ΚΡ|ΚΩΔ|ΛΟΓ|Μ|ΜΕΡ|ΜΟΝΑΔ|ΜΟΥΛ|ΜΟΥΣ|ΜΠΑΓΙΑΤ|ΜΠΑΝ|ΜΠΟΛ|ΜΠΟΣ|ΜΥΣΤ|Ν|ΝΙΤ|ΞΙΚ|ΟΠΤ|ΠΑΝ|ΠΕΤΣ|ΠΙΚΑΝΤ|ΠΙΤΣ|ΠΛΑΣΤ|ΠΛΙΑΤΣ|ΠΟΝΤ|ΠΟΣΤΕΛΝ|ΠΡΩΤΟΔ|ΣΕΡΤ|ΣΗΜΑΝΤ|ΣΤΑΤ|ΣΥΝΑΔ|ΣΥΝΟΜΗΛ|ΤΕΛ|ΤΕΧΝ|ΤΡΟΠ|ΤΣΑΜ|ΥΠΟΔ|Φ|ΦΙΛΟΝ|ΦΥΛΟΔ|ΦΥΣ|ΧΑΣ)$/.test(match[1]) || /(ΦΟΙΝ)$/.test(match[1])) {
            stemmedWord += 'ΙΚ';
          }
        }

        //step 5
        //5a
        if (stemmedWord === 'ΑΓΑΜΕ') {
          stemmedWord = 'ΑΓΑΜ';
        }
        if ((match = /^(.+?)(ΑΓΑΜΕ|ΗΣΑΜΕ|ΟΥΣΑΜΕ|ΗΚΑΜΕ|ΗΘΗΚΑΜΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
        }
        if ((match = /^(.+?)(ΑΜΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(ΑΝΑΠ|ΑΠΟΘ|ΑΠΟΚ|ΑΠΟΣΤ|ΒΟΥΒ|ΞΕΘ|ΟΥΛ|ΠΕΘ|ΠΙΚΡ|ΠΟΤ|ΣΙΧ|Χ)$/.test(match[1])) {
            stemmedWord += 'ΑΜ';
          }
        }

        //5b
        if ((match = /^(.+?)(ΑΓΑΝΕ|ΗΣΑΝΕ|ΟΥΣΑΝΕ|ΙΟΝΤΑΝΕ|ΙΟΤΑΝΕ|ΙΟΥΝΤΑΝΕ|ΟΝΤΑΝΕ|ΟΤΑΝΕ|ΟΥΝΤΑΝΕ|ΗΚΑΝΕ|ΗΘΗΚΑΝΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(ΤΡ|ΤΣ)$/.test(match[1])) {
            stemmedWord += 'ΑΓΑΝ';
          }
        }
        if ((match = /^(.+?)(ΑΝΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (endsInVowel2(stemmedWord) || /^(ΒΕΤΕΡ|ΒΟΥΛΚ|ΒΡΑΧΜ|Γ|ΔΡΑΔΟΥΜ|Θ|ΚΑΛΠΟΥΖ|ΚΑΣΤΕΛ|ΚΟΡΜΟΡ|ΛΑΟΠΛ|ΜΩΑΜΕΘ|Μ|ΜΟΥΣΟΥΛΜΑΝ|ΟΥΛ|Π|ΠΕΛΕΚ|ΠΛ|ΠΟΛΙΣ|ΠΟΡΤΟΛ|ΣΑΡΑΚΑΤΣ|ΣΟΥΛΤ|ΤΣΑΡΛΑΤ|ΟΡΦ|ΤΣΙΓΓ|ΤΣΟΠ|ΦΩΤΟΣΤΕΦ|Χ|ΨΥΧΟΠΛ|ΑΓ|ΟΡΦ|ΓΑΛ|ΓΕΡ|ΔΕΚ|ΔΙΠΛ|ΑΜΕΡΙΚΑΝ|ΟΥΡ|ΠΙΘ|ΠΟΥΡΙΤ|Σ|ΖΩΝΤ|ΙΚ|ΚΑΣΤ|ΚΟΠ|ΛΙΧ|ΛΟΥΘΗΡ|ΜΑΙΝΤ|ΜΕΛ|ΣΙΓ|ΣΠ|ΣΤΕΓ|ΤΡΑΓ|ΤΣΑΓ|Φ|ΕΡ|ΑΔΑΠ|ΑΘΙΓΓ|ΑΜΗΧ|ΑΝΙΚ|ΑΝΟΡΓ|ΑΠΗΓ|ΑΠΙΘ|ΑΤΣΙΓΓ|ΒΑΣ|ΒΑΣΚ|ΒΑΘΥΓΑΛ|ΒΙΟΜΗΧ|ΒΡΑΧΥΚ|ΔΙΑΤ|ΔΙΑΦ|ΕΝΟΡΓ|ΘΥΣ|ΚΑΠΝΟΒΙΟΜΗΧ|ΚΑΤΑΓΑΛ|ΚΛΙΒ|ΚΟΙΛΑΡΦ|ΛΙΒ|ΜΕΓΛΟΒΙΟΜΗΧ|ΜΙΚΡΟΒΙΟΜΗΧ|ΝΤΑΒ|ΞΗΡΟΚΛΙΒ|ΟΛΙΓΟΔΑΜ|ΟΛΟΓΑΛ|ΠΕΝΤΑΡΦ|ΠΕΡΗΦ|ΠΕΡΙΤΡ|ΠΛΑΤ|ΠΟΛΥΔΑΠ|ΠΟΛΥΜΗΧ|ΣΤΕΦ|ΤΑΒ|ΤΕΤ|ΥΠΕΡΗΦ|ΥΠΟΚΟΠ|ΧΑΜΗΛΟΔΑΠ|ΨΗΛΟΤΑΒ)$/.test(match[1])) {
            stemmedWord += 'ΑΝ';
          }
        }

        //5c
        if ((match = /^(.+?)(ΗΣΕΤΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
        }

        if ((match = /^(.+?)(ΕΤΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (endsInVowel2(stemmedWord) || /(ΟΔ|ΑΙΡ|ΦΟΡ|ΤΑΘ|ΔΙΑΘ|ΣΧ|ΕΝΔ|ΕΥΡ|ΤΙΘ|ΥΠΕΡΘ|ΡΑΘ|ΕΝΘ|ΡΟΘ|ΣΘ|ΠΥΡ|ΑΙΝ|ΣΥΝΔ|ΣΥΝ|ΣΥΝΘ|ΧΩΡ|ΠΟΝ|ΒΡ|ΚΑΘ|ΕΥΘ|ΕΚΘ|ΝΕΤ|ΡΟΝ|ΑΡΚ|ΒΑΡ|ΒΟΛ|ΩΦΕΛ)$/.test(match[1]) || /^(ΑΒΑΡ|ΒΕΝ|ΕΝΑΡ|ΑΒΡ|ΑΔ|ΑΘ|ΑΝ|ΑΠΛ|ΒΑΡΟΝ|ΝΤΡ|ΣΚ|ΚΟΠ|ΜΠΟΡ|ΝΙΦ|ΠΑΓ|ΠΑΡΑΚΑΛ|ΣΕΡΠ|ΣΚΕΛ|ΣΥΡΦ|ΤΟΚ|Υ|Δ|ΕΜ|ΘΑΡΡ|Θ)$/.test(match[1])) {
            stemmedWord += 'ΕΤ';
          }
        }

        //5d
        if ((match = /^(.+?)(ΟΝΤΑΣ|ΩΝΤΑΣ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^ΑΡΧ$/.test(match[1])) {
            stemmedWord += 'ΟΝΤ';
          }
          if (/ΚΡΕ$/.test(match[1])) {
            stemmedWord += 'ΩΝΤ';
          }
        }

        //5e
        if ((match = /^(.+?)(ΟΜΑΣΤΕ|ΙΟΜΑΣΤΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^ΟΝ$/.test(match[1])) {
            stemmedWord += 'ΟΜΑΣΤ';
          }
        }

        //5f
        if ((match = /^(.+?)(ΙΕΣΤΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(Π|ΑΠ|ΣΥΜΠ|ΑΣΥΜΠ|ΑΚΑΤΑΠ|ΑΜΕΤΑΜΦ)$/.test(match[1])) {
            stemmedWord += 'ΙΕΣΤ';
          }
        }

        if ((match = /^(.+?)(ΕΣΤΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(ΑΛ|ΑΡ|ΕΚΤΕΛ|Ζ|Μ|Ξ|ΠΑΡΑΚΑΛ|ΠΡΟ|ΝΙΣ)$/.test(match[1])) {
            stemmedWord += 'ΕΣΤ';
          }
        }

        //5g
        if ((match = /^(.+?)(ΗΘΗΚΑ|ΗΘΗΚΕΣ|ΗΘΗΚΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
        }

        if ((match = /^(.+?)(ΗΚΑ|ΗΚΕΣ|ΗΚΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/(ΣΚΩΛ|ΣΚΟΥΛ|ΝΑΡΘ|ΣΦ|ΟΘ|ΠΙΘ)$/.test(match[1]) || /^(ΔΙΑΘ|Θ|ΠΑΡΑΚΑΤΑΘ|ΠΡΟΣΘ|ΣΥΝΘ)$/.test(match[1])) {
            stemmedWord += 'ΗΚ';
          }
        }

        //5h
        if ((match = /^(.+?)(ΟΥΣΑ|ΟΥΣΕΣ|ΟΥΣΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (endsInVowel(stemmedWord) || /^(ΦΑΡΜΑΚ|ΧΑΔ|ΑΓΚ|ΑΝΑΡΡ|ΒΡΟΜ|ΕΚΛΙΠ|ΛΑΜΠΙΔ|ΛΕΧ|Μ|ΠΑΤ|Ρ|Λ|ΜΕΔ|ΜΕΣΑΖ|ΥΠΟΤΕΙΝ|ΑΜ|ΑΙΘ|ΑΝΗΚ|ΔΕΣΠΟΖ|ΕΝΔΙΑΦΕΡ)$/.test(match[1]) || /(ΠΟΔΑΡ|ΒΛΕΠ|ΠΑΝΤΑΧ|ΦΡΥΔ|ΜΑΝΤΙΛ|ΜΑΛΛ|ΚΥΜΑΤ|ΛΑΧ|ΛΗΓ|ΦΑΓ|ΟΜ|ΠΡΩΤ)$/.test(match[1])) {
            stemmedWord += 'ΟΥΣ';
          }
        }

        //5i
        if ((match = /^(.+?)(ΑΓΑ|ΑΓΕΣ|ΑΓΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(ΑΒΑΣΤ|ΠΟΛΥΦ|ΑΔΗΦ|ΠΑΜΦ|Ρ|ΑΣΠ|ΑΦ|ΑΜΑΛ|ΑΜΑΛΛΙ|ΑΝΥΣΤ|ΑΠΕΡ|ΑΣΠΑΡ|ΑΧΑΡ|ΔΕΡΒΕΝ|ΔΡΟΣΟΠ|ΞΕΦ|ΝΕΟΠ|ΝΟΜΟΤ|ΟΛΟΠ|ΟΜΟΤ|ΠΡΟΣΤ|ΠΡΟΣΩΠΟΠ|ΣΥΜΠ|ΣΥΝΤ|Τ|ΥΠΟΤ|ΧΑΡ|ΑΕΙΠ|ΑΙΜΟΣΤ|ΑΝΥΠ|ΑΠΟΤ|ΑΡΤΙΠ|ΔΙΑΤ|ΕΝ|ΕΠΙΤ|ΚΡΟΚΑΛΟΠ|ΣΙΔΗΡΟΠ|Λ|ΝΑΥ|ΟΥΛΑΜ|ΟΥΡ|Π|ΤΡ|Μ)$/.test(match[1]) || (/(ΟΦ|ΠΕΛ|ΧΟΡΤ|ΛΛ|ΣΦ|ΡΠ|ΦΡ|ΠΡ|ΛΟΧ|ΣΜΗΝ)$/.test(match[1]) && !/^(ΨΟΦ|ΝΑΥΛΟΧ)$/.test(match[1])) || /(ΚΟΛΛ)$/.test(match[1])) {
            stemmedWord += 'ΑΓ';
          }
        }

        //5j
        if ((match = /^(.+?)(ΗΣΕ|ΗΣΟΥ|ΗΣΑ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(Ν|ΧΕΡΣΟΝ|ΔΩΔΕΚΑΝ|ΕΡΗΜΟΝ|ΜΕΓΑΛΟΝ|ΕΠΤΑΝ|Ι)$/.test(match[1])) {
            stemmedWord += 'ΗΣ';
          }
        }

        //5k
        if ((match = /^(.+?)(ΗΣΤΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(ΑΣΒ|ΣΒ|ΑΧΡ|ΧΡ|ΑΠΛ|ΑΕΙΜΝ|ΔΥΣΧΡ|ΕΥΧΡ|ΚΟΙΝΟΧΡ|ΠΑΛΙΜΨ)$/.test(match[1])) {
            stemmedWord += 'ΗΣΤ';
          }
        }

        //5l
        if ((match = /^(.+?)(ΟΥΝΕ|ΗΣΟΥΝΕ|ΗΘΟΥΝΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(Ν|Ρ|ΣΠΙ|ΣΤΡΑΒΟΜΟΥΤΣ|ΚΑΚΟΜΟΥΤΣ|ΕΞΩΝ)$/.test(match[1])) {
            stemmedWord += 'ΟΥΝ';
          }
        }

        //5m
        if ((match = /^(.+?)(ΟΥΜΕ|ΗΣΟΥΜΕ|ΗΘΟΥΜΕ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1];
          if (/^(ΠΑΡΑΣΟΥΣ|Φ|Χ|ΩΡΙΟΠΛ|ΑΖ|ΑΛΛΟΣΟΥΣ|ΑΣΟΥΣ)$/.test(match[1])) {
            stemmedWord += 'ΟΥΜ';
          }
        }

        //step 6
        //6a
        if ((match = /^(.+?)(ΜΑΤΟΙ|ΜΑΤΟΥΣ|ΜΑΤΟ|ΜΑΤΑ|ΜΑΤΩΣ|ΜΑΤΩΝ|ΜΑΤΟΣ|ΜΑΤΕΣ|ΜΑΤΗ|ΜΑΤΗΣ|ΜΑΤΟΥ)$/.exec(stemmedWord)) != null) {
          stemmedWord = match[1] + 'Μ';
          if (/^(ΓΡΑΜ)$/.test(match[1])) {
            stemmedWord += 'Α';
          } else if (/^(ΓΕ|ΣΤΑ)$/.test(match[1])) {
            stemmedWord += 'ΑΤ';
          }
        }

        //6b
        if ((match = /^(.+?)(ΟΥΑ)$/.exec(stemmedWord)) !== null) {
          stemmedWord = match[1] + 'ΟΥ';
        }

        //Handle long words
        if (word.length === stemmedWord.length) {
          if ((match = /^(.+?)(Α|ΑΓΑΤΕ|ΑΓΑΝ|ΑΕΙ|ΑΜΑΙ|ΑΝ|ΑΣ|ΑΣΑΙ|ΑΤΑΙ|ΑΩ|Ε|ΕΙ|ΕΙΣ|ΕΙΤΕ|ΕΣΑΙ|ΕΣ|ΕΤΑΙ|Ι|ΙΕΜΑΙ|ΙΕΜΑΣΤΕ|ΙΕΤΑΙ|ΙΕΣΑΙ|ΙΕΣΑΣΤΕ|ΙΟΜΑΣΤΑΝ|ΙΟΜΟΥΝ|ΙΟΜΟΥΝΑ|ΙΟΝΤΑΝ|ΙΟΝΤΟΥΣΑΝ|ΙΟΣΑΣΤΑΝ|ΙΟΣΑΣΤΕ|ΙΟΣΟΥΝ|ΙΟΣΟΥΝΑ|ΙΟΤΑΝ|ΙΟΥΜΑ|ΙΟΥΜΑΣΤΕ|ΙΟΥΝΤΑΙ|ΙΟΥΝΤΑΝ|Η|ΗΔΕΣ|ΗΔΩΝ|ΗΘΕΙ|ΗΘΕΙΣ|ΗΘΕΙΤΕ|ΗΘΗΚΑΤΕ|ΗΘΗΚΑΝ|ΗΘΟΥΝ|ΗΘΩ|ΗΚΑΤΕ|ΗΚΑΝ|ΗΣ|ΗΣΑΝ|ΗΣΑΤΕ|ΗΣΕΙ|ΗΣΕΣ|ΗΣΟΥΝ|ΗΣΩ|Ο|ΟΙ|ΟΜΑΙ|ΟΜΑΣΤΑΝ|ΟΜΟΥΝ|ΟΜΟΥΝΑ|ΟΝΤΑΙ|ΟΝΤΑΝ|ΟΝΤΟΥΣΑΝ|ΟΣ|ΟΣΑΣΤΑΝ|ΟΣΑΣΤΕ|ΟΣΟΥΝ|ΟΣΟΥΝΑ|ΟΤΑΝ|ΟΥ|ΟΥΜΑΙ|ΟΥΜΑΣΤΕ|ΟΥΝ|ΟΥΝΤΑΙ|ΟΥΝΤΑΝ|ΟΥΣ|ΟΥΣΑΝ|ΟΥΣΑΤΕ|Υ||ΥΑ|ΥΣ|Ω|ΩΝ|ΟΙΣ)$/.exec(stemmedWord)) !== null) {
            stemmedWord = match[1];
          }
        }

        //step 7
        if ((match = /^(.+?)(ΕΣΤΕΡ|ΕΣΤΑΤ|ΟΤΕΡ|ΟΤΑΤ|ΥΤΕΡ|ΥΤΑΤ|ΩΤΕΡ|ΩΤΑΤ)$/.exec(stemmedWord)) != null) {
          if (!/^(ΕΞ|ΕΣ|ΑΝ|ΚΑΤ|Κ|ΠΡ)$/.test(match[1])) {
            stemmedWord = match[1];
          }
          if (/^(ΚΑ|Μ|ΕΛΕ|ΛΕ|ΔΕ)$/.test(match[1])) {
            stemmedWord += 'ΥΤ';
          }
        }

        return stemmedWord;
      }

      return function(token) {
        if (typeof token.update === "function") {
          return token.update(function(word) {
            return stem(word.toUpperCase()).toLowerCase();
          });
        } else {
          return stem(token.toUpperCase()).toLowerCase();
        }
      }
    })();

    lunr.Pipeline.registerFunction(lunr.el.stemmer, 'stemmer-el');

    /* lunr stopWordFilter function */
    lunr.el.stopWordFilter = lunr.generateStopWordFilter('αλλα αν αντι απο αυτα αυτεσ αυτη αυτο αυτοι αυτοσ αυτουσ αυτων για δε δεν εαν ειμαι ειμαστε ειναι εισαι ειστε εκεινα εκεινεσ εκεινη εκεινο εκεινοι εκεινοσ εκεινουσ εκεινων ενω επι η θα ισωσ κ και κατα κι μα με μετα μη μην να ο οι ομωσ οπωσ οσο οτι παρα ποια ποιεσ ποιο ποιοι ποιοσ ποιουσ ποιων που προσ πωσ σε στη στην στο στον τα την τησ το τον τοτε του των ωσ'.split(' '));

    lunr.Pipeline.registerFunction(lunr.el.stopWordFilter, 'stopWordFilter-el');

    /* lunr normilizer function */
    lunr.el.normilizer = (function() {
      var accentMap = {
        "Ά": "Α",
        "ά": "α",
        "Έ": "Ε",
        "έ": "ε",
        "Ή": "Η",
        "ή": "η",
        "Ί": "Ι",
        "ί": "ι",
        "Ό": "Ο",
        "ο": "ο",
        "Ύ": "Υ",
        "ύ": "υ",
        "Ώ": "Ω",
        "ώ": "ω",
        "Ϊ": "Ι",
        "ϊ": "ι",
        "Ϋ": "Υ",
        "ϋ": "υ",
        "ΐ": "ι",
        "ΰ": "υ"
      };

      return function(token) {
        if (typeof token.update === "function") {
          return token.update(function(term) {
            var ret = "";
            for (var i = 0; i < term.length; i++) {
              ret += accentMap[term.charAt(i)] || term.charAt(i);
            }
            return ret;
          });
        } else {
          var ret = "";
          for (var i = 0; i < token.length; i++) {
            ret += accentMap[token.charAt(i)] || token.charAt(i);
          }
          return ret;
        }
      }
    })();

    lunr.Pipeline.registerFunction(lunr.el.normilizer, 'normilizer-el');
  };
}))