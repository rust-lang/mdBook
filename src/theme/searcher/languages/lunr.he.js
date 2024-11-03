/*!
 * Lunr languages, `Hebrew` language
 * https://github.com/avisaradir/lunr-languages-he
 *
 * Copyright 2023, Adir Avisar
 * http://www.mozilla.org/MPL/
 */
/*!
 * based on
 * Kazem Taghva, Rania Elkhoury, and Jeffrey Coombs (2005)
 * Meryeme Hadni, Abdelmonaime Lachkar, and S. Alaoui Ouatik (2012)
 *
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
    lunr.he = function() {
      this.pipeline.reset();
      this.pipeline.add(
        lunr.he.trimmer,
        lunr.he.stopWordFilter,
        lunr.he.stemmer
      );

      // for lunr version 2
      // this is necessary so that every searched word is also stemmed before
      // in lunr <= 1 this is not needed, as it is done using the normal pipeline
      if (this.searchPipeline) {
        this.searchPipeline.reset();
        this.searchPipeline.add(lunr.he.stemmer)
      }
    };

    /* lunr trimmer function */
    lunr.he.wordCharacters = "\u0590-\u05FF\u05D0-\u05EAa-zA-Zａ-ｚＡ-Ｚ0-9０-９";
    lunr.he.trimmer = lunr.trimmerSupport.generateTrimmer(lunr.he.wordCharacters);

    lunr.Pipeline.registerFunction(lunr.he.trimmer, 'trimmer-he');

    /* lunr stemmer function */
    lunr.he.stemmer = (function() {
      var self = this;
      var word = '';
      self.result = false;

      self.execArray = [
        'cleanWord'
      ];

      self.stem = function() {
        var counter = 0;
        self.result = false;
        while (counter < self.execArray.length && self.result != true) {
          self.result = self[self.execArray[counter]]();
          counter++;
        }
      }

      self.setCurrent = function(word) {
        self.word = word;
      }

      self.getCurrent = function() {
        return self.word
      }

      /*remove elongating character and test that the word does not contain non-hebrew characters.
      If the word contains special characters, don't stem. */
      self.cleanWord = function() {
        var wordCharacters = "\u0591-\u05F4\u05D0-\u05EA";
        var testRegex = new RegExp("[^" + wordCharacters + "]");
        if (testRegex.test(word)) {
          return true;
        }
        return false;
      }

      /* and return a function that stems a word for the current locale */
      return function(token) {
        // for lunr version 2
        if (typeof token.update === "function") {
          return token.update(function(word) {
            self.setCurrent(word);
            self.stem();
            return self.getCurrent();
          })
        } else { // for lunr version <= 1
          self.setCurrent(token);
          self.stem();
          return self.getCurrent();
        }

      }
    })();

    lunr.Pipeline.registerFunction(lunr.he.stemmer, 'stemmer-he');

    lunr.he.stopWordFilter = lunr.generateStopWordFilter('אבל או אולי אותו אותי אותך אותם אותן אותנו אז אחר אחרות אחרי אחריכן אחרים אחרת אי איזה איך אין איפה אל אלה אלו אם אנחנו אני אף אפשר את אתה אתכם אתכן אתם אתן באיזה באיזו בגלל בין בלבד בעבור בעזרת בכל בכן בלי במידה במקום שבו ברוב בשביל בשעה ש בתוך גם דרך הוא היא היה היי היכן היתה היתי הם הן הנה הסיבה שבגללה הרי ואילו ואת זאת זה זות יהיה יוכל יוכלו יותר מדי יכול יכולה יכולות יכולים יכל יכלה יכלו יש כאן כאשר כולם כולן כזה כי כיצד כך כל כלל כמו כן כפי כש לא לאו לאיזותך לאן לבין לה להיות להם להן לו לזה לזות לי לך לכם לכן למה למעלה למעלה מ למטה למטה מ למעט למקום שבו למרות לנו לעבר לעיכן לפיכך לפני מאד מאחורי מאיזו סיבה מאין מאיפה מבלי מבעד מדוע מה מהיכן מול מחוץ מי מידע מכאן מכל מכן מלבד מן מנין מסוגל מעט מעטים מעל מצד מקום בו מתחת מתי נגד נגר נו עד עז על עלי עליו עליה עליהם עליך עלינו עם עצמה עצמהם עצמהן עצמו עצמי עצמם עצמן עצמנו פה רק שוב של שלה שלהם שלהן שלו שלי שלך שלכה שלכם שלכן שלנו שם תהיה תחת'.split(' '));

    lunr.Pipeline.registerFunction(lunr.he.stopWordFilter, 'stopWordFilter-he');
  };
}))