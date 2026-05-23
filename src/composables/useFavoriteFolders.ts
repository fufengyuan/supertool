import { ref } from 'vue';

const FAVORITE_KEY = 'hermes-favorite-folders';

/**
 * 收藏文件夹管理
 */
export function useFavoriteFolders() {
  const favoriteFolders = ref<string[]>([]);

  const loadFavoriteFolders = () => {
    try {
      const saved = localStorage.getItem(FAVORITE_KEY);
      if (saved) {
        favoriteFolders.value = JSON.parse(saved);
      }
    } catch {
      favoriteFolders.value = [];
    }
  };

  const saveFavoriteFolders = () => {
    try {
      localStorage.setItem(FAVORITE_KEY, JSON.stringify(favoriteFolders.value));
    } catch {
      // 忽略保存失败
    }
  };

  const removeFavoriteFolder = (folder: string) => {
    favoriteFolders.value = favoriteFolders.value.filter(f => f !== folder);
    saveFavoriteFolders();
  };

  const addFavoriteFolder = (folder: string) => {
    if (!favoriteFolders.value.includes(folder)) {
      favoriteFolders.value.push(folder);
      saveFavoriteFolders();
    }
  };

  return {
    favoriteFolders,
    loadFavoriteFolders,
    saveFavoriteFolders,
    removeFavoriteFolder,
    addFavoriteFolder,
  };
}