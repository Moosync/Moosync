package app.moosync.filescanner

import android.annotation.SuppressLint
import android.content.Context
import android.content.ContentUris
import android.database.Cursor
import android.provider.MediaStore
import android.util.Log
import java.io.FileNotFoundException
import Song
import Artist
import Album
import Genre
import android.media.MediaScannerConnection
import getPathFromURI
import getUriFromID


class AudioScanner {
    private val TAG = "file-scanner"

    fun readDirectory(mContext: Context): ArrayList<Song> {
        val songList: ArrayList<Song> = ArrayList()

        val proj = arrayListOf(
            MediaStore.Audio.Media._ID,
            MediaStore.Audio.Media.DISPLAY_NAME,
            MediaStore.Audio.Media.ALBUM,
            MediaStore.Audio.Media.ARTIST,
            MediaStore.Audio.Media.ALBUM_ID,
            MediaStore.Audio.Media.ARTIST_ID,
            MediaStore.Audio.Media.DURATION,
            MediaStore.Audio.Media.IS_MUSIC,
            MediaStore.Audio.Media.DATE_MODIFIED,
        )

        if (android.os.Build.VERSION.SDK_INT >= 30) {
            proj.add(MediaStore.Audio.Media.GENRE)
            proj.add(MediaStore.Audio.Media.GENRE_ID)
        }

        MediaScannerConnection.scanFile(mContext, arrayOf("/storage/emulated/0"), null, null);

        val cursor = mContext.contentResolver.query(
            MediaStore.Audio.Media.EXTERNAL_CONTENT_URI,
            proj.toTypedArray(),
            null,
            null,
            MediaStore.Audio.Media.DEFAULT_SORT_ORDER
        )

        Log.d("file-scanner", "readDirectory: got cursor, moving over elems")

        if (cursor != null && cursor.moveToFirst()) {
            do {
                val isMusic =
                    cursor.getInt(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.IS_MUSIC))

                Log.d("file-scanner", "readDirectory: file is music: ${isMusic}")
                if (isMusic != 0) {
                    try {
                        val id = cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media._ID))
                        Log.d(TAG, "readDirectory: got id $id")
                        songList.add(
                            Song(
                                title = cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DISPLAY_NAME)),
                                duration = cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DURATION)) / 1000,
                                path = getPathFromURI(mContext, ContentUris.withAppendedId(MediaStore.Audio.Media.EXTERNAL_CONTENT_URI, id)),
                                artist = getArtist(cursor),
                                album = getAlbum(mContext, id, cursor),
                                genre = getGenre(cursor),
                                playbackUrl = id.toString(),
                                song_coverPath_high = getUriFromID(mContext, id),
                                song_coverPath_low = getUriFromID(mContext, id),
                                type = "LOCAL"
                            )
                        )
                        Log.d(TAG, "readDirectory: added $id to song list")
                    } catch (e: Exception) {
                        Log.e(TAG, "readDirectory: ", e)
                    }
                }
            } while (cursor.moveToNext())

            Log.d(TAG, "readDirectory: closing cursor")
            cursor.close()
        }

        Log.d(TAG, "readDirectory: returning song list");
        return songList
    }

    private fun getArtist(cursor: Cursor): List<Artist>? {
        val artistId =
            cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ARTIST_ID))
        val artistName =
            cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ARTIST))

        if (artistId != 0L) {
            return listOf(Artist(artistName, null))
        }
        return null
    }

    private fun getAlbum(context: Context, id: Long, cursor: Cursor): Album? {
        val albumId = cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ALBUM_ID))
        val albumName = cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ALBUM))

        if (albumId != 0L) {
            return Album(albumName, getUriFromID(context, id), getUriFromID(context, id))
        }
        return null
    }

    @SuppressLint("InlinedApi")
    private fun getGenre(cursor: Cursor): List<Genre>? {
        val genreIdIndex = cursor.getColumnIndex(MediaStore.Audio.Media.GENRE_ID)
        if (genreIdIndex >= 0) {
            val genreId = cursor.getLong(genreIdIndex)
            val genreName =
                cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.GENRE))

            if (genreId != 0L) {
                return listOf(Genre(genreName))
            }
        }
        return null
    }
}
