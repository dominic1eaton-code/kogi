import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubCommunityShowcaseComponent } from './hub-community-showcase.component';

describe('HubCommunityShowcaseComponent', () => {
  let component: HubCommunityShowcaseComponent;
  let fixture: ComponentFixture<HubCommunityShowcaseComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubCommunityShowcaseComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubCommunityShowcaseComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
