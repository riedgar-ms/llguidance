const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

if (!fs.existsSync('node_modules/marked/package.json')) {
    console.log('Installing dependencies...');
    execSync('npm install', { stdio: 'inherit' });
    console.log('Dependencies installed.');
}

const crypto = require('crypto');
const marked = require('marked');
const hljs = require('highlight.js');

const outDir = 'tmp';
const inputPath = process.argv[2];

if (!inputPath) {
    console.error('Usage: node to_html.js <input_file.md>');
    process.exit(1);
}

fs.mkdirSync(outDir, { recursive: true });

let input = fs.readFileSync(inputPath, 'utf8');

// Replace Mermaid blocks with inline SVGs
let count = 0;
let processed = '';
let rest = input;

while (true) {
    const start = rest.indexOf('```mermaid');
    if (start === -1) {
        processed += rest;
        break;
    }

    const end = rest.indexOf('```', start + 9);
    if (end === -1) throw new Error('Unclosed mermaid block');

    const before = rest.slice(0, start);
    const code = rest.slice(start + 10, end).trim();
    rest = rest.slice(end + 3);

    const hash = crypto.createHash('sha256').update(code).digest('hex').slice(0, 8);
    const baseName = `mermaid_${hash}`;
    const mmdFile = path.join(outDir, `${baseName}.mmd`);
    const svgFile = path.join(outDir, `${baseName}.svg`);

    if (!fs.existsSync(svgFile)) {
        console.log(`Generating SVG for Mermaid block ${count + 1}...`);
        fs.writeFileSync(mmdFile, code, 'utf8');
        execSync(`node_modules/.bin/mmdc -i ${mmdFile} -o ${svgFile} -c mermaid.json`);
        fs.unlinkSync(mmdFile);
    }

    const svgContent = fs.readFileSync(svgFile, 'utf8');
    processed += before + `\n\n${svgContent}\n\n`;
    count++;
}

hljs.registerLanguage('lark', function (hljs) {
    return {
        name: 'Lark',
        aliases: ['lark'],
        keywords: {
            keyword: ['?'],
        },
        contains: [
            hljs.COMMENT('//', '$'),
            hljs.COMMENT('#', '$'),
            {
                className: 'string',
                begin: /"/, end: /"/,
                contains: [hljs.BACKSLASH_ESCAPE]
            },
            {
                className: 'symbol',
                begin: /\/(?!\/)(?:\\.|[^\\\/\n])+?\//,
            },
            {
                className: 'symbol',
                begin: /^[a-z_][a-z0-9_]*\s*:/i, // rule: ...
                returnBegin: true,
                contains: [
                    {
                        className: 'title',
                        begin: /^[a-z_][a-z0-9_]*/i
                    }
                ]
            },
            {
                className: 'keyword',
                begin: /\b[A-Z][A-Z0-9_]*\b/ // TERMINALS
            },
            {
                className: 'operator',
                begin: /->|:|\*|\+|\?|\||\(|\)/
            },
            {
                className: 'meta',
                begin: /%[a-zA-Z_]+/
            }
        ]
    };
});

// Custom renderer with highlight.js
const renderer = {
    code(obj) {
        const language = obj.lang || 'plaintext';
        const code = obj.text || '';
        const validLang = hljs.getLanguage(language) ? language : 'plaintext';
        const { value: highlighted } = hljs.highlight(code, { language: validLang });
        return `<pre><code class="hljs ${validLang}">${highlighted}</code></pre>`;
    }
};

marked.use({ renderer });

// Inline external SVG image references
processed = processed.replace(/!\[.*?\]\((.+?\.svg)\)/g, (match, svgPath) => {
    const absPath = path.resolve(path.dirname(inputPath), svgPath);
    if (!fs.existsSync(absPath)) {
        console.warn(`Warning: SVG file not found: ${svgPath}`);
        return match;
    }
    const svgContent = fs.readFileSync(absPath, 'utf8');
    return `\n\n${svgContent}\n\n`;
});

const bodyHtml = marked.parse(processed);

const titleMatch = input.match(/^#\s*(.*)$/m);
const title = titleMatch ? titleMatch[1] : 'Rendered Markdown';

const fullHtmlTemplate = `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>@title@</title>

<!-- https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.css -->
<style>
pre code.hljs{display:block;overflow-x:auto;padding:1em}code.hljs{padding:3px 5px}
.hljs{color:#24292e;background:#fff}.hljs-doctag,.hljs-keyword,.hljs-meta .hljs-keyword,.hljs-template-tag,.hljs-template-variable,.hljs-type,.hljs-variable.language_{color:#d73a49}.hljs-title,.hljs-title.class_,.hljs-title.class_.inherited__,.hljs-title.function_{color:#6f42c1}.hljs-attr,.hljs-attribute,.hljs-literal,.hljs-meta,.hljs-number,.hljs-operator,.hljs-selector-attr,.hljs-selector-class,.hljs-selector-id,.hljs-variable{color:#005cc5}
.hljs-meta .hljs-string,.hljs-regexp,.hljs-string{
 color:rgb(3, 98, 6)
 }
.hljs-built_in,.hljs-symbol{color:#e36209}.hljs-code,.hljs-comment,.hljs-formula{color:#6a737d}.hljs-name,.hljs-quote,.hljs-selector-pseudo,.hljs-selector-tag{color:#22863a}.hljs-subst{color:#24292e}.hljs-section{color:#005cc5;font-weight:700}.hljs-bullet{color:#735c0f}.hljs-emphasis{color:#24292e;font-style:italic}.hljs-strong{color:#24292e;font-weight:700}.hljs-addition{color:#22863a;background-color:#f0fff4}.hljs-deletion{color:#b31d28;background-color:#ffeef0}
</style>

<style>

  body {
    max-width: 800px;
    margin: auto;
    font-family: 'Georgia', serif;
    font-size: 18px;
    line-height: 1.7;
    color: #1c1c1c;
    background: #fff;
    padding: 0em 1em;
  }

  @media print {
    body {
        font-size: 14px;
    }
    @page {
        margin: 2cm;
    }
   }

  h1, h2, h3, h4, h5 {
    font-family: 'Helvetica Neue', sans-serif;
    font-weight: 600;
    line-height: 1.3;
    margin-top: 2em;
    margin-bottom: 1em;
  }

  p {
    margin: 1.5em 0;
  }

  pre {
    background: #f4f4f4;
    border-left: 4px solid #ccc;
    padding: 0em;
    overflow-x: auto;
    line-height: 1.5;
  }

  code {
    background: #f4f4f4;
    padding: 0.2em 0.4em;
    border-radius: 4px;
    font-family: 'Consolas', 'SF Mono', 'Menlo', monospace;
    font-weight: 400;
    font-size: 80%;
  }

  pre code.hljs {
    padding: 0em 1em;
  }

  img, svg {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 2em auto;
  }

  blockquote {
    border-left: 4px solid #ddd;
    padding-left: 1em;
    color: #555;
    font-style: italic;
    margin: 1.5em 0;
  }

  a {
    color: #1a0dab;
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }
</style>
</head>
<body>
@bodyHtml@
</body>
</html>`;

console.log(`Processed ${count} Mermaid block(s)`);

function saveHTML(path, template, replacements) {
    const data = template.replace(/@(\w+)@/g, (_, key) => {
        return replacements[key] || '';
    });

    fs.writeFileSync(path, data, 'utf8');
    const kb = Math.round(Buffer.byteLength(data, 'utf8') / 1024);
    console.log(`HTML saved to ${path}; ${kb} KB`);
}

saveHTML('llg-brr.html', fullHtmlTemplate, {
    title,
    bodyHtml
});


const indexTitle = "LLGuidance"
const indexBody = `
<h1>${indexTitle}</h1>
<p>
Please checkout the <a href="https://github.com/guidance-ai/llguidance">GitHub repository</a> for more information,
or read the blog entry <a href="llg-brr.html">${title}</a>.
</p>
`

saveHTML('index.html', fullHtmlTemplate, {
    title,
    bodyHtml: indexBody
});
