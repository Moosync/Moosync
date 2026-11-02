package app.moosync.android.services

import android.util.Log
import app.moosync.android.MainActivity
import org.junit.After
import org.junit.AfterClass
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.BeforeClass
import org.junit.Test
import org.mockito.MockedStatic
import org.mockito.Mockito
import java.io.File

class MediaServiceLifecycleTest {

    companion object {
        private var mockedLog: MockedStatic<Log>? = null

        @JvmStatic
        @BeforeClass
        fun setUpClass() {
            mockedLog = Mockito.mockStatic(Log::class.java)
        }

        @JvmStatic
        @AfterClass
        fun tearDownClass() {
            mockedLog?.close()
        }
    }

    private lateinit var tempDir: File

    @Before
    fun setUp() {
        tempDir = File.createTempFile("moosync_test", "")
        tempDir.delete()
        tempDir.mkdirs()
    }

    @After
    fun tearDown() {
        tempDir.deleteRecursively()
        MoosyncService.instance = null
        MoosyncService.isTaskRemoved = false
        MoosyncService.isNotificationActive = false
        MainActivity.isActivityAlive = false
        MainActivity.hasRunBefore = false
    }

    @Test
    fun testResolveThumbnailDirectFile() {
        val sampleFile = File(tempDir, "cover.png").apply { writeText("imgdata") }
        val filesDir = File(tempDir, "files").apply { mkdirs() }

        val resolved = MediaSessionHandler.resolveThumbnailFile(filesDir, sampleFile.absolutePath)

        assertNotNull(resolved)
        assertEquals(sampleFile.absolutePath, resolved?.absolutePath)
    }

    @Test
    fun testResolveThumbnailWithDataDataPrefixMappedToAppDataDir() {
        val appDataDir = File(tempDir, "app_data").apply { mkdirs() }
        val filesDir = File(appDataDir, "files").apply { mkdirs() }
        val thumbsDir = File(appDataDir, "cache/thumbnails").apply { mkdirs() }
        val thumbFile = File(thumbsDir, "abc123hash.png").apply { writeText("imagedata") }

        val resolved = MediaSessionHandler.resolveThumbnailFile(
            filesDir,
            "/data/data/app.moosync.android/cache/thumbnails/abc123hash.png"
        )

        assertNotNull(resolved)
        assertEquals(thumbFile.canonicalPath, resolved?.canonicalPath)
    }

    @Test
    fun testResolveThumbnailWithDataUserPrefixMappedToAppDataDir() {
        val appDataDir = File(tempDir, "app_data").apply { mkdirs() }
        val filesDir = File(appDataDir, "files").apply { mkdirs() }
        val thumbsDir = File(appDataDir, "cache/thumbnails").apply { mkdirs() }
        val thumbFile = File(thumbsDir, "user0hash.png").apply { writeText("imagedata") }

        val resolved = MediaSessionHandler.resolveThumbnailFile(
            filesDir,
            "/data/user/0/app.moosync.android/cache/thumbnails/user0hash.png"
        )

        assertNotNull(resolved)
        assertEquals(thumbFile.canonicalPath, resolved?.canonicalPath)
    }

    @Test
    fun testResolveThumbnailNonExistentReturnsNull() {
        val filesDir = File(tempDir, "files").apply { mkdirs() }

        val resolved = MediaSessionHandler.resolveThumbnailFile(filesDir, "/non/existent/path/thumb.png")

        assertNull(resolved)
    }

    @Test
    fun testKillAppOnlyWhenBothTaskRemovedAndNotificationCleared() {
        // App is swiped from recents, but notification is still active -> do NOT kill
        assertFalse(MoosyncService.shouldKillApp(taskRemoved = true, notificationActive = true))

        // Notification is dismissed, but app is in recents / active -> do NOT kill
        assertFalse(MoosyncService.shouldKillApp(taskRemoved = false, notificationActive = false))

        // Normal playing state in recents -> do NOT kill
        assertFalse(MoosyncService.shouldKillApp(taskRemoved = false, notificationActive = true))

        // Both task removed AND notification cleared -> MUST kill
        assertTrue(MoosyncService.shouldKillApp(taskRemoved = true, notificationActive = false))
    }

    @Test
    fun testMainActivityTracksLifecycleFlags() {
        assertFalse(MainActivity.hasRunBefore)
        assertFalse(MainActivity.isActivityAlive)

        MainActivity.hasRunBefore = true
        MainActivity.isActivityAlive = true

        assertTrue(MainActivity.hasRunBefore)
        assertTrue(MainActivity.isActivityAlive)
    }
}
