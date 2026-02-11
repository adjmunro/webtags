/**
 * Convert between Chrome bookmarks and JSON API v1.1 format
 */

import type {
  ChromeBookmark,
  BookmarksData,
  BookmarkResource,
  TagResource,
  Resource,
} from './types';

/**
 * Extract bookmarks from Chrome bookmark tree
 */
export function extractBookmarks(node: ChromeBookmark): ChromeBookmark[] {
  const bookmarks: ChromeBookmark[] = [];

  if (node.url) {
    // This is a bookmark (has a URL)
    bookmarks.push(node);
  }

  if (node.children) {
    // Recursively process children
    for (const child of node.children) {
      bookmarks.push(...extractBookmarks(child));
    }
  }

  return bookmarks;
}

/**
 * Parse tags from bookmark title or folder structure
 * For now, we extract tags from the bookmark title using #tag syntax
 * e.g., "My Website #programming #rust" -> tags: ["programming", "rust"]
 */
export function extractTagsFromTitle(title: string): {
  cleanTitle: string;
  tags: string[];
} {
  const tagRegex = /#(\w+)/g;
  const tags: string[] = [];
  let match;

  while ((match = tagRegex.exec(title)) !== null) {
    tags.push(match[1]);
  }

  // Remove tags from title
  const cleanTitle = title.replace(tagRegex, '').trim();

  return { cleanTitle, tags };
}

/**
 * Convert Chrome bookmarks to JSON API v1.1 format
 */
export function chromeToJsonApi(
  chromeBookmarks: ChromeBookmark[]
): BookmarksData {
  const data: BookmarkResource[] = [];
  const tagsMap = new Map<string, TagResource>();

  // Process each bookmark
  for (const chromeBookmark of chromeBookmarks) {
    if (!chromeBookmark.url) continue;

    const { cleanTitle, tags: tagNames } = extractTagsFromTitle(
      chromeBookmark.title
    );

    // Create tag resources for new tags
    const tagIds: string[] = [];
    for (const tagName of tagNames) {
      if (!tagsMap.has(tagName)) {
        const tagId = `tag-${tagName.toLowerCase()}`;
        tagsMap.set(tagName, {
          type: 'tag',
          id: tagId,
          attributes: {
            name: tagName,
          },
        });
      }
      const tag = tagsMap.get(tagName)!;
      tagIds.push(tag.id);
    }

    // Create bookmark resource
    const bookmark: BookmarkResource = {
      type: 'bookmark',
      id: chromeBookmark.id,
      attributes: {
        url: chromeBookmark.url,
        title: cleanTitle,
        created: chromeBookmark.dateAdded
          ? new Date(chromeBookmark.dateAdded).toISOString()
          : new Date().toISOString(),
      },
    };

    if (tagIds.length > 0) {
      bookmark.relationships = {
        tags: {
          data: tagIds.map((id) => ({ type: 'tag', id })),
        },
      };
    }

    data.push(bookmark);
  }

  return {
    jsonapi: { version: '1.1' },
    data,
    included: Array.from(tagsMap.values()),
  };
}

/**
 * Apply JSON API bookmarks to Chrome bookmarks
 * This updates the browser bookmark tree based on the JSON data
 */
export async function applyToChrome(
  bookmarksData: BookmarksData
): Promise<void> {
  // Get all existing Chrome bookmarks
  const [tree] = await chrome.bookmarks.getTree();
  const existingBookmarks = extractBookmarks(tree);
  const existingBookmarkIds = new Set(
    existingBookmarks.map((b) => b.id)
  );

  // Create a map of tags
  const tagsById = new Map<string, TagResource>();
  for (const resource of bookmarksData.included || []) {
    if (resource.type === 'tag') {
      tagsById.set(resource.id, resource as TagResource);
    }
  }

  // Process bookmarks from JSON
  for (const resource of bookmarksData.data) {
    if (resource.type !== 'bookmark') continue;

    const bookmark = resource as BookmarkResource;

    // Get tags for this bookmark
    const tagNames: string[] = [];
    if (bookmark.relationships?.tags?.data) {
      for (const tagRef of bookmark.relationships.tags.data) {
        const tag = tagsById.get(tagRef.id);
        if (tag) {
          tagNames.push(tag.attributes.name);
        }
      }
    }

    // Build title with tags
    const titleWithTags =
      tagNames.length > 0
        ? `${bookmark.attributes.title} ${tagNames.map((t) => `#${t}`).join(' ')}`
        : bookmark.attributes.title;

    if (existingBookmarkIds.has(bookmark.id)) {
      // Update existing bookmark
      try {
        await chrome.bookmarks.update(bookmark.id, {
          title: titleWithTags,
          url: bookmark.attributes.url,
        });
      } catch (error) {
        console.error(`Failed to update bookmark ${bookmark.id}:`, error);
      }
    } else {
      // Create new bookmark
      try {
        // Get the "Other Bookmarks" folder (usually id "2" in Chrome)
        const otherBookmarks = await chrome.bookmarks.search({ title: 'Other Bookmarks' });
        const parentId = otherBookmarks.length > 0 ? otherBookmarks[0].id : '1';

        await chrome.bookmarks.create({
          parentId,
          title: titleWithTags,
          url: bookmark.attributes.url,
        });
      } catch (error) {
        console.error(`Failed to create bookmark:`, error);
      }
    }
  }

  // Remove bookmarks that don't exist in JSON
  const jsonBookmarkIds = new Set(
    bookmarksData.data
      .filter((r) => r.type === 'bookmark')
      .map((r) => r.id)
  );

  for (const chromeBookmark of existingBookmarks) {
    if (!jsonBookmarkIds.has(chromeBookmark.id)) {
      try {
        await chrome.bookmarks.remove(chromeBookmark.id);
      } catch (error) {
        console.error(`Failed to remove bookmark ${chromeBookmark.id}:`, error);
      }
    }
  }
}

/**
 * Export all Chrome bookmarks to JSON API format
 */
export async function exportChromeBookmarks(): Promise<BookmarksData> {
  const [tree] = await chrome.bookmarks.getTree();
  const bookmarks = extractBookmarks(tree);
  return chromeToJsonApi(bookmarks);
}
