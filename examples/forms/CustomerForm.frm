VERSION 5.00
Begin {C62A69F0-16DC-11CE-9E98-00AA00574A4F} CustomerForm
 Caption = "Customer details"
 ClientWidth = 6000
 ClientHeight = 4000
 OleObjectBlob = "CustomerForm.9af671fcecf553d3.frx":0000
End
Attribute VB_Name = "CustomerForm"
Option Explicit

Private Sub CommandButton1_Click()
    Dim n As Long
    n=7
    Stop
    Debug.Print n
    Me.Hide
End Sub

Private Sub TextBox1_Change()
    Label1.Caption=TextBox1.Text
End Sub
