package com.assistant;

import com.google.gson.Gson;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Document;
import org.jsoup.nodes.Element;
import org.jsoup.nodes.Node;
import org.jsoup.nodes.TextNode;
import org.jsoup.select.Elements;

import java.util.ArrayList;
import java.util.List;

public class ScraperCLI {
    private static final String USER_AGENT = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/117.0";
    private static final int TIMEOUT_MS = 8000;
    private static final int MAX_CHAR_LIMIT = 2500; 

    static class SearchResult {
        String title;
        String url;
        String content;
        String error;

        public SearchResult(String title, String url, String content, String error) {
            this.title = title;
            this.url = url;
            this.content = content;
            this.error = error;
        }
    }

    public static void main(String[] args) {
        if (args.length < 1) {
            System.err.println("Usage: java -jar scraper.jar \"keyword phrase\" [max_results]");
            System.exit(1);
        }

        String query = args[0];
        int maxResults = args.length > 1 ? Integer.parseInt(args[1]) : 3;

        List<SearchResult> results = new ArrayList<>();
        Gson gson = new Gson();

        try {
            Document searchDoc = Jsoup.connect("https://lite.duckduckgo.com/lite/")
                    .data("q", query)
                    .userAgent(USER_AGENT)
                    .timeout(TIMEOUT_MS)
                    .post();

            Elements links = searchDoc.select("a");

            int count = 0;
            for (Element link : links) {
                if (count >= maxResults) break;

                String targetUrl = link.attr("href");
                String title = link.text();

                if (!targetUrl.startsWith("http") || targetUrl.contains("duckduckgo.com") || title.length() < 5) {
                    continue;
                }

                String mdContent = scrapeAndConvertToMarkdown(targetUrl);
                
                if (mdContent.length() > 100) {
                    results.add(new SearchResult(title, targetUrl, mdContent, null));
                    count++;
                }
                
                Thread.sleep(300); 
            }
        } catch (Exception e) {
            results.add(new SearchResult(null, null, null, "Search failed: " + e.getMessage()));
        }

        System.out.println(gson.toJson(results));
    }

    private static String scrapeAndConvertToMarkdown(String url) {
        try {
            Document doc = Jsoup.connect(url).userAgent(USER_AGENT).timeout(TIMEOUT_MS).get();
            
            // 1. Run the Readability filter
            removeBoilerplate(doc);

            // 2. Select main content container
            Element root = findMainContainer(doc);

            // 3. Convert HTML DOM to Markdown
            StringBuilder md = new StringBuilder();
            domToMarkdown(root, md);

            // 4. Clean up whitespace
            String cleaned = md.toString()
                    .replaceAll("\n{3,}", "\n\n")
                    .replaceAll("[ \t]+", " ")
                    .trim();

            // 5. Regex cleanup for dangling boilerplate lines
            cleaned = cleaned.replaceAll("(?im)^(skip to( main)? content|privacy policy|terms of( service| use)|all rights reserved|cookie policy|read more|leave a comment).*$\n*", "");

            // 6. Truncate cleanly
            if (cleaned.length() > MAX_CHAR_LIMIT) {
                int safeEnd = cleaned.lastIndexOf('.', MAX_CHAR_LIMIT);
                if (safeEnd > 500) {
                    cleaned = cleaned.substring(0, safeEnd + 1) + "\n\n[...truncated for brevity...]";
                } else {
                    cleaned = cleaned.substring(0, MAX_CHAR_LIMIT) + "...";
                }
            }

            return cleaned.trim();

        } catch (Exception e) {
            return "[Extraction Failed: " + e.getMessage() + "]";
        }
    }

    private static void removeBoilerplate(Document doc) {
        // A. Remove structural noise tags
        doc.select("script, style, noscript, nav, footer, header, aside, iframe, svg, form, button, dialog").remove();

        // B. Remove common noise by class/id substring
        String[] junkKeywords = {"cookie", "banner", "menu", "sidebar", "social", "share", "comment", "popup", "modal", "advert", "promo", "related", "newsletter", "subscribe"};
        for (String word : junkKeywords) {
            doc.select("[class*=" + word + "], [id*=" + word + "]").remove();
        }

        // C. Link Density Filter: Destroy hidden navigation menus / link farms
        for (Element block : doc.select("ul, div, section")) {
            Elements blockLinks = block.select("a");
            if (blockLinks.size() >= 4) { 
                float totalLength = block.text().length();
                if (totalLength > 0) {
                    float linkLength = 0;
                    for (Element link : blockLinks) {
                        linkLength += link.text().length();
                    }
                    // If more than 40% of the text is hyperlinks, it's a menu or related articles grid
                    if ((linkLength / totalLength) > 0.40) {
                        block.remove();
                    }
                }
            }
        }
    }

    private static Element findMainContainer(Document doc) {
        Elements article = doc.select("article");
        if (!article.isEmpty()) return article.first();

        Elements main = doc.select("main");
        if (!main.isEmpty()) return main.first();

        Elements body = doc.select(".post-content, .article-content, .entry-content, #content, .content");
        if (!body.isEmpty()) return body.first();

        return doc.body();
    }

    private static void domToMarkdown(Node node, StringBuilder out) {
        if (node instanceof TextNode) {
            String text = ((TextNode) node).text().trim();
            if (!text.isEmpty()) {
                out.append(text).append(" ");
            }
            return;
        }

        if (node instanceof Element) {
            Element el = (Element) node;
            String tag = el.tagName().toLowerCase();

            switch (tag) {
                case "h1":
                    out.append("\n\n# ");
                    break;
                case "h2":
                    out.append("\n\n## ");
                    break;
                case "h3":
                case "h4":
                case "h5":
                case "h6":
                    out.append("\n\n### ");
                    break;
                case "p":
                    out.append("\n\n");
                    break;
                case "li":
                    out.append("\n- ");
                    break;
                case "tr":
                    out.append("\n| ");
                    break;
            }

            for (Node child : el.childNodes()) {
                domToMarkdown(child, out);
                if (tag.equals("th") || tag.equals("td")) {
                    out.append(" | ");
                }
            }

            switch (tag) {
                case "strong":
                case "b":
                    out.append("** ");
                    break;
                case "em":
                case "i":
                    out.append("* ");
                    break;
            }
        }
    }
}