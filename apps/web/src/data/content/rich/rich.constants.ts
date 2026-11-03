/* @layer renderer-app @kind constants */
const LINE_BREAK = /\r?\n/;

const UNKNOWN_IMAGE = ' names an image this record does not have. Add the image first, then use the number it gets.';

const LEAD_LATE =
  ': only the first line may be plain text; write the rest as bullets (- point), nested at most two levels, or as images ![caption](img:n).';

export { LEAD_LATE, LINE_BREAK, UNKNOWN_IMAGE };
