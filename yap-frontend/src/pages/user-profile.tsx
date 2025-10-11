import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useOutletContext } from 'react-router-dom'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { TopPageLayout } from '@/components/TopPageLayout'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { get_profile_by_id, update_profile } from '../../../yap-frontend-rs/pkg'
import { Pencil, X, Check } from 'lucide-react'
import { toast } from 'sonner'

interface Profile {
  id: string
  display_name: string | null
  bio: string | null
  display_name_slug: string | null
  notifications_enabled: boolean
  created_at: string
  updated_at: string
}

type OutletContextType = {
  userInfo: { id: string; email: string } | undefined
  accessToken: string | undefined
}

export function UserProfilePage() {
  const { id } = useParams<{ id: string }>()
  const { userInfo, accessToken } = useOutletContext<OutletContextType>()
  const navigate = useNavigate()
  const [profile, setProfile] = useState<Profile | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isEditing, setIsEditing] = useState(false)
  const [editDisplayName, setEditDisplayName] = useState('')
  const [editBio, setEditBio] = useState('')
  const [saving, setSaving] = useState(false)

  const isOwnProfile = userInfo?.id === id

  useEffect(() => {
    async function loadProfile() {
      if (!id) {
        setError('No user ID provided')
        setLoading(false)
        return
      }

      try {
        setLoading(true)
        const profileData = await get_profile_by_id(id)
        setProfile(profileData as Profile)
        setEditDisplayName(profileData.display_name || '')
        setEditBio(profileData.bio || '')
        setError(null)
      } catch (err) {
        console.error('Error loading profile:', err)
        setError('Failed to load profile')
      } finally {
        setLoading(false)
      }
    }

    loadProfile()
  }, [id])

  const handleSave = async () => {
    if (!accessToken) {
      toast.error('You must be logged in to edit your profile')
      return
    }

    try {
      setSaving(true)
      await update_profile(
        editDisplayName || null,
        editBio || null,
        accessToken
      )

      // Reload profile to get updated data including slug
      const profileData = await get_profile_by_id(id!)
      setProfile(profileData as Profile)
      setIsEditing(false)
      toast.success('Profile updated successfully')
    } catch (err) {
      console.error('Error updating profile:', err)
      toast.error('Failed to update profile')
    } finally {
      setSaving(false)
    }
  }

  const handleCancel = () => {
    setEditDisplayName(profile?.display_name || '')
    setEditBio(profile?.bio || '')
    setIsEditing(false)
  }

  if (loading) {
    return (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: 'Profile', onBack: () => navigate('/') }
        }}
      >
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <Skeleton className="h-8 w-48" />
            </CardHeader>
            <CardContent className="space-y-3">
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-3/4" />
            </CardContent>
          </Card>
        </div>
      </TopPageLayout>
    )
  }

  if (error || !profile) {
    return (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: 'Profile', onBack: () => navigate('/') }
        }}
      >
        <Card>
          <CardContent className="pt-6">
            <div className="text-center text-muted-foreground">
              {error || 'Profile not found'}
            </div>
          </CardContent>
        </Card>
      </TopPageLayout>
    )
  }

  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: { label: 'Profile', onBack: () => navigate('/') }
      }}
    >
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <div className="flex items-start justify-between">
              <div className="flex-1">
                {isEditing ? (
                  <div className="space-y-3">
                    <div className="flex flex-col gap-2">
                      <Label htmlFor="displayName">Display Name</Label>
                      <Input
                        id="displayName"
                        value={editDisplayName}
                        onChange={(e) => setEditDisplayName(e.target.value)}
                        placeholder="Enter your display name"
                        maxLength={50}
                      />
                    </div>
                  </div>
                ) : (
                  <>
                    <CardTitle>
                      {profile.display_name || 'Anonymous User'}
                    </CardTitle>
                    {profile.display_name_slug && (
                      <p className="text-sm text-muted-foreground">
                        @{profile.display_name_slug}
                      </p>
                    )}
                  </>
                )}
              </div>
              {isOwnProfile && !isEditing && (
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setIsEditing(true)}
                  className="ml-2"
                >
                  <Pencil className="h-4 w-4" />
                </Button>
              )}
              {isOwnProfile && isEditing && (
                <div className="flex gap-2 ml-2">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleCancel}
                    disabled={saving}
                  >
                    <X className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="default"
                    size="icon"
                    onClick={handleSave}
                    disabled={saving}
                  >
                    <Check className="h-4 w-4" />
                  </Button>
                </div>
              )}
            </div>
          </CardHeader>
          <CardContent>
            {isEditing ? (
              <div className="flex flex-col gap-2">
                <Label htmlFor="bio">Bio</Label>
                <Textarea
                  id="bio"
                  value={editBio}
                  onChange={(e) => setEditBio(e.target.value)}
                  placeholder="Tell us about yourself"
                  rows={4}
                  maxLength={500}
                />
                <p className="text-xs text-muted-foreground mt-1">
                  {editBio.length}/500 characters
                </p>
              </div>
            ) : (
              <>
                {profile.bio ? (
                  <p className="text-foreground whitespace-pre-wrap">{profile.bio}</p>
                ) : (
                  <p className="text-muted-foreground italic">No bio yet</p>
                )}
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Member Since</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">
              {new Date(profile.created_at).toLocaleDateString('en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric'
              })}
            </p>
          </CardContent>
        </Card>
      </div>
    </TopPageLayout>
  )
}
