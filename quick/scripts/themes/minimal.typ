// Theme: minimal
// Clean black-and-white layout. No watermark, no tinted backgrounds.
// Headings use weight/size only — no colored fills. Best for formal submissions.
#let accent  = rgb("#1a1a1a")
#let muted   = rgb("#777777")

#let grid-images(..imgs) = {
  v(0.3em)
  align(center, block(width: 100%,
    grid(
      columns: (1fr, 1fr),
      gutter: 0.5em,
      ..imgs.pos().map(img =>
        block(stroke: 0.5pt + rgb("#cccccc"), radius: 2pt, clip: true)[#img]
      )
    )
  ))
  v(0.3em)
}

// Captioned image — used by cmarker for every markdown image.
#let captioned-image(src, alt: none, ..args) = {
  let has-alt = alt != none and str(alt).trim() != ""
  figure(
    image(src, ..args),
    caption: {
      set text(size: 8pt)
      if has-alt {
        emph(alt)
        h(0.4em)
        text(fill: rgb("#888888"))[· #raw(src)]
      } else {
        text(fill: rgb("#888888"))[#raw(src)]
      }
    },
  )
}

#let project-name = "Laem Chabang House"
#let build-date   = datetime.today().display("[day] [month repr:long] [year]")

#let conf(
  title: none,
  subtitle: none,
  authors: (),
  keywords: (),
  date: none,
  abstract-title: none,
  abstract: none,
  thanks: none,
  cols: 1,
  margin: (x: 2.0cm, top: 2.2cm, bottom: 2.4cm),
  paper: "a4",
  lang: "en",
  region: "US",
  font: none,
  fontsize: 10pt,
  mathfont: none,
  codefont: none,
  linestretch: 1.4,
  sectionnumbering: none,
  linkcolor: none,
  citecolor: none,
  filecolor: none,
  pagenumbering: "1",
  status: "Draft",
  rev: "1",
  doc,
) = {

  set document(
    title: if title != none { title } else { project-name },
    author: project-name,
  )

  set page(
    paper: paper,
    margin: margin,
    header: context {
      if counter(page).get().first() > 1 {
        set text(size: 8pt, fill: muted)
        grid(
          columns: (1fr, 1fr),
          align(left)[#project-name — #title],
          align(right)[#build-date],
        )
        line(length: 100%, stroke: 0.4pt + muted)
      }
    },
    footer: context {
      set text(size: 8pt, fill: muted)
      line(length: 100%, stroke: 0.4pt + muted)
      grid(
        columns: (1fr, 1fr, 1fr),
        align(left)[#title],
        align(center)[Rev #rev — #status],
        align(right)[#counter(page).display("1 / 1", both: true)],
      )
    },
    // No background watermark in minimal theme
  )

  set text(
    font: ("Inter", "Noto Sans", "Noto Sans Thai"),
    size: fontsize,
    lang: lang,
    hyphenate: false,
  )

  set par(leading: 0.65em, spacing: 1.0em)

  // Cover: plain title block, no color fill
  if title != none {
    v(1em)
    text(size: 22pt, weight: "bold")[#title]
    v(0.2em)
    set text(size: 9pt, fill: muted)
    [#project-name · #build-date · Rev #rev — #status]
    v(0.3em)
    line(length: 100%, stroke: 1.5pt + black)
    v(0.8em)
  }

  // H1 — bold rule only, no filled block
  show heading.where(level: 1): it => {
    v(0.8em)
    text(size: 13pt, weight: "bold")[#it.body]
    v(0.15em)
    line(length: 100%, stroke: 1.5pt + black)
    v(0.3em)
  }

  // H2
  show heading.where(level: 2): it => {
    v(0.5em)
    text(size: 10.5pt, weight: "bold")[#it.body]
    line(length: 100%, stroke: 0.6pt + muted)
    v(0.2em)
  }

  // H3
  show heading.where(level: 3): it => {
    v(0.4em)
    text(size: 10pt, weight: "bold", fill: muted)[#it.body]
    v(0.15em)
  }

  // Tables — no color fill, simple dividers
  set table(
    inset: (x: 0.5em, y: 0.35em),
    stroke: (_, y) => if y == 0 { (bottom: 1pt + black) }
                      else { (top: 0.4pt + rgb("#dddddd")) },
    fill: none,
  )
  show table.cell: set text(size: 8.5pt)
  show table.cell.where(y: 0): set text(weight: "bold", size: 8.5pt)
  show table: set block(width: 100%)

  show link: underline.with(stroke: 0.4pt + rgb("#333333"))
  show link: set text(fill: rgb("#333333"))

  show figure: it => {
    v(0.3em)
    align(center, block(width: 100%)[
      #block(stroke: 0.4pt + rgb("#cccccc"), radius: 2pt, clip: true)[#it.body]
      #if it.caption != none [
        #v(0.2em)
        #set par(leading: 0.4em)
        #it.caption
      ]
    ])
    v(0.5em)
  }

  show line: set line(stroke: 0.4pt + rgb("#cccccc"))

  doc
}
